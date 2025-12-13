use std::{collections::HashSet, sync::Arc};

use axum::{
  body::Body,
  extract::{Path, Query, State},
  response::{IntoResponse, Response},
};
use base64::Engine;
use http::{StatusCode, header};
use os_model::entities::{
  clients,
  jwks::{get_jwk_for_sign_and_verify, list_jwks},
  revoked_tokens,
  users::{self, get_user_client_by_client_id},
};
use sha2::{Digest, Sha256};
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  core::{config::AppConfig, encryption::verify_password, helper::json_to_string_vec},
  router::{
    Form, Json,
    common::{
      constants::{
        SCOPE_ADDRESS, SCOPE_EMAIL, SCOPE_OFFLINE, SCOPE_OPENID, SCOPE_PHONE, SCOPE_PROFILE,
        TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE, TOKEN_ISSUE_TYPE_PASSWORD,
        TOKEN_ISSUE_TYPE_REFRESH_TOKEN, TOKEN_TYPE_AUTHORIZATION_CODE, TOKEN_TYPE_REFRESH,
      },
      entity::{AuthorizationCodeClaims, BasicClaims, OpenIdClaims, Token},
      helper::create_user_token,
      permissions::Permission,
    },
    entity::RouterState,
    error::{
      CREDENTIALS, HttpError, INTERNAL_ERROR, INVALID_ERROR, NOT_ALLOWED_ERROR, NOT_FOUND_ERROR,
      NOT_SUPPORTED_ERROR, REQUIRED_ERROR,
    },
    middleware::{
      authorization::{Authorization, parse_authorization},
      user_authorization::UserAuthorization,
    },
    oidc::{
      constants::{
        ALWAYS_ALLOWED_GRANT_TYPES, GRANT_TYPE_AUTHORIZATION_CODE, GRANT_TYPE_PASSWORD,
        GRANT_TYPE_REFRESH_TOKEN, GRANT_TYPE_REVOKE, TAG,
      },
      entity::{
        AuthorizeRequest, Client, ClientAllowed, ClientAuthentication, ClientRegisterRequest,
        EndSessionRequest, JWK, JWKs, OpenIdConfiguration, RevokeRequest, TokenRequest,
      },
    },
  },
};

#[utoipa::path(
  get,
  path = "/.well-known/jwks.json",
  tags = [TAG],
  responses(
    (status = 200, description = "JSON Web Keys", body = JWKs),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn jwks(State(state): State<RouterState>) -> impl IntoResponse {
  let jwks = match list_jwks(&state.database).await {
    Ok(jwks) => jwks,
    Err(e) => {
      log::error!("error getting jwks: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let keys = jwks
    .into_iter()
    .map(|jwk| JWK {
      key_ops: jwk.public_key_operations(),

      kid: jwk.kid.to_string(),
      kty: jwk.kty,
      alg: jwk.alg,
      r#use: jwk.r#use,

      n: jwk.n,
      e: jwk.e,

      crv: jwk.crv,
      x: jwk.x,
      y: jwk.y,

      x5c: jwk.x5c,
      x5u: jwk.x5u,
      x5t: jwk.x5t,
      x5t_s256: jwk.x5t_s256,
    })
    .collect();

  axum::Json(JWKs { keys }).into_response()
}

#[utoipa::path(
  get,
  path = "/.well-known/openid-configuration",
  tags = [TAG],
  responses(
    (status = 200, description = "OpenId Configuration", body = OpenIdConfiguration),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn openid_configuration(State(state): State<RouterState>) -> impl IntoResponse {
  let api_url = state.config.api_url();

  let jwks = match list_jwks(&state.database).await {
    Ok(jwks) => jwks,
    Err(e) => {
      log::error!("error getting jwks: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let mut signing_algs = HashSet::new();

  for jwk in jwks {
    let key_operations = jwk.key_operations();

    log::info!("{:?}", key_operations);

    if key_operations
      .iter()
      .find(|s| s.as_str() == "sign")
      .is_some()
    {
      signing_algs.insert(jwk.alg.to_owned());
    }
  }

  let mut response_modes_supported = vec![
    "query".to_owned(),
    "fragment".to_owned(),
    "form_post".to_owned(),
  ];

  if state.config.ui_url.is_some() {
    response_modes_supported.push("web_message".to_owned());
  }

  axum::Json(OpenIdConfiguration {
    authorization_endpoint: format!("{}/authorize", api_url),
    device_authorization_endpoint: format!("{}/device-authorize", api_url),
    token_endpoint: format!("{}/token", api_url),
    end_session_endpoint: Some(format!("{}/end-session", api_url)),
    userinfo_endpoint: Some(format!("{}/user-info", api_url)),
    revocation_endpoint: Some(format!("{}/revoke", api_url)),
    registration_endpoint: Some(format!("{}/register-client", api_url)),
    jwks_uri: format!("{}/.well-known/jwks.json", api_url),
    response_types_supported: vec!["code".to_owned(), "none".to_owned()],
    response_modes_supported: response_modes_supported,
    subject_types_supported: vec!["public".to_owned(), "pairwise".to_owned()],
    id_token_signing_alg_values_supported: signing_algs.into_iter().collect(),
    scopes_supported: vec![
      SCOPE_OPENID.to_owned(),
      SCOPE_PROFILE.to_owned(),
      SCOPE_ADDRESS.to_owned(),
      SCOPE_OFFLINE.to_owned(),
      SCOPE_EMAIL.to_owned(),
      SCOPE_PHONE.to_owned(),
    ],
    token_endpoint_auth_methods_supported: vec![
      "client_secret_post".to_owned(),
      "client_secret_basic".to_owned(),
      "client_secret_jwt".to_owned(),
      "private_key_jwt".to_owned(),
      "none".to_owned(),
    ],
    claims_supported: vec![
      "sub".to_owned(),
      "aud".to_owned(),
      "exp".to_owned(),
      "iat".to_owned(),
      "iss".to_owned(),
      "name".to_owned(),
      "family_name".to_owned(),
      "given_name".to_owned(),
      "email".to_owned(),
      "email_verified".to_owned(),
      "phone".to_owned(),
      "phone_verified".to_owned(),
    ],
    code_challenge_methods_supported: vec!["S256".to_owned()],
    grant_types_supported: vec![
      "password".to_owned(),
      "authorization_code".to_owned(),
      "refresh_token".to_owned(),
    ],
    issuer: api_url,
  })
  .into_response()
}

#[utoipa::path(
  get,
  path = "/end-session",
  tags = [TAG],
  params(EndSessionRequest),
  responses(
    (status = 204, description = "Session ended"),
    (status = 401, description = "Unauthorized Error", body = HttpError),
    (status = 403, description = "Forbiddon Error", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn end_session(
  State(state): State<RouterState>,
  Query(end_session_request): Query<EndSessionRequest>,
) -> impl IntoResponse {
  let client_id = match end_session_request.client_id {
    Some(client_id) => client_id,
    None => match end_session_request.id_token_hint {
      Some(id_token_hint) => {
        match parse_authorization::<BasicClaims>(&state.database, &state.config, &id_token_hint)
          .await
        {
          Ok((token, _jwk)) => token.claims.aud,
          Err(e) => {
            log::error!("failed to parse id_token_hint: {}", e);
            return e.into_response();
          }
        }
      }
      None => {
        return HttpError::bad_request()
          .with_error("client_id", INVALID_ERROR)
          .with_error("id_token_hint", INVALID_ERROR)
          .into_response();
      }
    },
  };

  let client_model = match clients::get_client_by_client_id(&state.database, &client_id).await {
    Ok(Some(client_model)) => client_model,
    Ok(None) => {
      return HttpError::not_found()
        .with_error("client", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("failed to fetch client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let post_logout_redirect_uri = match Url::parse(&end_session_request.post_logout_redirect_uri) {
    Ok(post_logout_redirect_uri) => post_logout_redirect_uri,
    Err(e) => {
      log::error!("invalid post_logout_redirect_uri: {}", e);
      return HttpError::bad_request()
        .with_error("post_logout_redirect_uri", INVALID_ERROR)
        .into_response();
    }
  };

  let post_logout_redirect_uri_string = if let Some(post_logout_redirect_uris) = client_model
    .post_logout_redirect_uris
    .as_ref()
    .map(json_to_string_vec)
  {
    let post_logout_redirect_uri_string =
      post_logout_redirect_uri.origin().ascii_serialization() + post_logout_redirect_uri.path();
    if !post_logout_redirect_uris.contains(&post_logout_redirect_uri_string) {
      return HttpError::bad_request()
        .with_error("post_logout_redirect_uri", NOT_ALLOWED_ERROR)
        .into_response();
    }
    post_logout_redirect_uri_string
  } else {
    return HttpError::bad_request()
      .with_error("client", INVALID_ERROR)
      .into_response();
  };

  match Response::builder()
    .status(StatusCode::FOUND)
    .header(header::LOCATION, post_logout_redirect_uri_string)
    .body(Body::empty())
  {
    Ok(response) => response.into_response(),
    Err(e) => {
      log::error!("Failed to build response: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  post,
  path = "/token",
  tags = [TAG],
  request_body(content = TokenRequest, content_type = "application/x-www-form-urlencoded"),
  responses(
    (status = 201, description = "Token returned", body = Token),
    (status = 401, description = "Unauthorized Error", body = HttpError),
    (status = 403, description = "Forbiddon Error", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn token(
  State(state): State<RouterState>,
  Form(token_request): Form<TokenRequest>,
) -> impl IntoResponse {
  let token_result = match token_request {
    TokenRequest::Password {
      scope,
      username,
      password,
      client_auth,
    } => password_grant(state, scope, username, password, client_auth).await,
    TokenRequest::RefreshToken {
      refresh_token,
      client_auth,
    } => refresh_token_grant(state, refresh_token, client_auth).await,
    TokenRequest::AuthorizationCode {
      code,
      code_verifier,
      client_auth,
    } => authorization_code_grant(state, code, code_verifier, client_auth).await,
  };
  match token_result {
    Ok(token) => axum::Json(token).into_response(),
    Err(e) => e.into_response(),
  }
}

async fn password_grant(
  state: RouterState,
  scope: String,
  username: String,
  password: String,
  client_auth: ClientAuthentication,
) -> Result<Token, HttpError> {
  let client_id = if let Some(client_id) = &client_auth.client_id {
    let _client_model = validate_client_authentication(
      &state.database,
      client_id,
      &client_auth,
      GRANT_TYPE_PASSWORD,
      &scope,
    )
    .await?;
    client_id.to_owned()
  } else {
    state.config.api_url()
  };

  let user = match users::get_user_by_username_or_primary_email(&state.database, &username).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized().with_error(CREDENTIALS, INVALID_ERROR)),
    Err(e) => {
      log::error!("error getting user: {}", e);
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
  };

  let user_password =
    match users::get_user_active_password_by_user_id(&state.database, user.id).await {
      Ok(Some(user_password)) => user_password,
      Ok(None) => {
        return Err(HttpError::forbidden().with_error(CREDENTIALS, TOKEN_ISSUE_TYPE_PASSWORD));
      }
      Err(e) => {
        log::error!("error fetching user password from database: {}", e);
        return Err(HttpError::unauthorized().with_application_error(INTERNAL_ERROR));
      }
    };

  match verify_password(&password, &user_password.encrypted_password) {
    Ok(true) => {}
    Ok(false) => {
      log::info!("invalid password for user id {}", user.id);
      return Err(HttpError::unauthorized().with_error(CREDENTIALS, INVALID_ERROR));
    }
    Err(e) => {
      log::error!("error verifying user password: {}", e);
      return Err(HttpError::unauthorized().with_application_error(INTERNAL_ERROR));
    }
  }

  let jwk = match get_jwk_for_sign_and_verify(&state.database).await {
    Ok(Some(jwk)) => jwk,
    Ok(None) => {
      log::error!("error no valid jwk for signing and verifying jwts");
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
    Err(e) => {
      log::error!("error getting jwk: {}", e);
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
  };

  create_user_token(
    &state.database,
    &state.config,
    jwk,
    user,
    client_id,
    scope,
    TOKEN_ISSUE_TYPE_PASSWORD.to_owned(),
  )
  .await
}

async fn refresh_token_grant(
  state: RouterState,
  refresh_token: String,
  client_auth: ClientAuthentication,
) -> Result<Token, HttpError> {
  let (token_data, jwk_model) =
    parse_authorization::<BasicClaims>(&state.database, &state.config, &refresh_token).await?;

  if token_data.claims.r#type != TOKEN_TYPE_REFRESH {
    return Err(HttpError::unauthorized());
  }

  let client_id = if let Some(client_id) = &client_auth.client_id {
    if client_id != &token_data.claims.aud {
      return Err(HttpError::unauthorized().with_error("client_id", INVALID_ERROR));
    }
    client_id.to_owned()
  } else {
    token_data.claims.aud.to_owned()
  };
  let _client_model = validate_client_authentication(
    &state.database,
    &client_id,
    &client_auth,
    GRANT_TYPE_REFRESH_TOKEN,
    &token_data.claims.scope,
  )
  .await?;

  let user_id = match token_data.claims.sub.parse::<i64>() {
    Ok(id) => id,
    Err(e) => {
      log::error!(
        "invalid refresh token sub claim is not a valid user id: {}",
        e
      );
      return Err(HttpError::unauthorized());
    }
  };

  let user = match users::get_user_by_id(&state.database, user_id).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized()),
    Err(e) => {
      log::error!("error fetching user: {}", e);
      return Err(HttpError::internal_error());
    }
  };

  create_user_token(
    &state.database,
    &state.config,
    jwk_model,
    user,
    client_id,
    token_data.claims.scope,
    TOKEN_ISSUE_TYPE_REFRESH_TOKEN.to_owned(),
  )
  .await
}

async fn authorization_code_grant(
  state: RouterState,
  code: String,
  code_verifier: Option<String>,
  client_auth: ClientAuthentication,
) -> Result<Token, HttpError> {
  let (token_data, jwk_model) =
    parse_authorization::<AuthorizationCodeClaims>(&state.database, &state.config, &code).await?;

  if token_data.claims.basic_claims.r#type != TOKEN_TYPE_AUTHORIZATION_CODE {
    return Err(HttpError::unauthorized());
  }

  let code_verifier = match code_verifier {
    Some(verifier) => verifier,
    None => {
      return Err(HttpError::bad_request().with_error("code_verifier", REQUIRED_ERROR));
    }
  };

  let computed_challenge = match token_data.claims.code_challenge_method.as_str() {
    "S256" => {
      let mut hasher = Sha256::new();
      hasher.update(code_verifier.as_bytes());
      let hash = hasher.finalize();
      base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
    }
    _ => {
      return Err(
        HttpError::bad_request().with_error("code_challenge_method", NOT_SUPPORTED_ERROR),
      );
    }
  };

  if computed_challenge != token_data.claims.code_challenge {
    return Err(HttpError::unauthorized().with_error("code_verifier", INVALID_ERROR));
  }

  let client_id = token_data.claims.basic_claims.aud;
  let _client_model = validate_client_authentication(
    &state.database,
    &client_id,
    &client_auth,
    GRANT_TYPE_AUTHORIZATION_CODE,
    &token_data.claims.basic_claims.scope,
  )
  .await?;

  let user_id = match token_data.claims.basic_claims.sub.parse::<i64>() {
    Ok(id) => id,
    Err(e) => {
      log::error!(
        "invalid authorization code sub claim is not a valid user id: {}",
        e
      );
      return Err(HttpError::unauthorized());
    }
  };

  let user = match users::get_user_by_id(&state.database, user_id).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized()),
    Err(e) => {
      log::error!("error fetching user: {}", e);
      return Err(HttpError::internal_error());
    }
  };

  create_user_token(
    &state.database,
    &state.config,
    jwk_model,
    user,
    client_id,
    token_data.claims.basic_claims.scope,
    TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE.to_owned(),
  )
  .await
}

pub(crate) async fn validate_client_authentication(
  db: &sea_orm::DatabaseConnection,
  client_id: &str,
  client_auth: &ClientAuthentication,
  grant_type: &str,
  scope: &str,
) -> Result<clients::Model, HttpError> {
  let client_model = match clients::get_client_by_client_id(db, client_id).await {
    Ok(Some(client)) => client,
    Ok(None) => {
      return Err(HttpError::not_found().with_error("client", NOT_FOUND_ERROR));
    }
    Err(e) => {
      log::error!("failed to fetch client: {}", e);
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
  };

  if !ALWAYS_ALLOWED_GRANT_TYPES.contains(&grant_type) {
    let grant_types_vec: Vec<String> = json_to_string_vec(&client_model.grant_types);
    if !grant_types_vec.contains(&grant_type.to_string()) {
      return Err(HttpError::bad_request().with_error("grant_type", NOT_ALLOWED_ERROR));
    }
  }

  if !client_model.is_active() {
    return Err(HttpError::forbidden().with_error("client", NOT_ALLOWED_ERROR));
  }

  let requested_scopes: HashSet<&str> = scope.split_whitespace().collect();
  let allowed_scopes_vec: Vec<String> = json_to_string_vec(&client_model.scopes);
  let allowed_scopes: HashSet<&str> = allowed_scopes_vec.iter().map(|s| s.as_str()).collect();

  for requested_scope in &requested_scopes {
    if !allowed_scopes.contains(requested_scope) {
      return Err(HttpError::bad_request().with_error("scope", NOT_ALLOWED_ERROR));
    }
  }

  match client_model.auth_method.as_str() {
    "client_secret_post" | "client_secret_basic" => {
      if let Some(auth_client_id) = &client_auth.client_id {
        if auth_client_id != client_id {
          return Err(HttpError::unauthorized().with_error("client_id", INVALID_ERROR));
        }
      }

      match &client_auth.client_secret {
        Some(provided_secret) => {
          if provided_secret != &client_model.client_secret {
            return Err(HttpError::unauthorized().with_error("client_secret", INVALID_ERROR));
          }
        }
        None => {
          return Err(HttpError::bad_request().with_error("client_secret", REQUIRED_ERROR));
        }
      }
    }
    "none" => {}
    "client_secret_jwt" | "private_key_jwt" => {
      log::warn!("JWT-based client authentication not yet implemented");
      return Err(HttpError::bad_request().with_error("auth_method", NOT_SUPPORTED_ERROR));
    }
    _ => {
      log::error!("unsupported auth_method: {}", client_model.auth_method);
      return Err(HttpError::bad_request().with_error("auth_method", NOT_ALLOWED_ERROR));
    }
  }

  Ok(client_model)
}

#[utoipa::path(
  get,
  path = "/authorize",
  tags = [TAG],
  params(AuthorizeRequest),
  responses(
    (status = 302, description = "Redirect"),
    (status = 400, description = "Application Error", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn authorize(
  State(state): State<RouterState>,
  Query(authorize_request): Query<AuthorizeRequest>,
) -> impl IntoResponse {
  authorize_internal(state.database, state.config, authorize_request).await
}

#[utoipa::path(
  post,
  path = "/authorize",
  tags = [TAG],
  request_body(content = AuthorizeRequest, content_type = "application/json"),
  responses(
    (status = 302, description = "Redirect"),
    (status = 400, description = "Application Error", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn post_authorize(
  State(state): State<RouterState>,
  Json(authorize_request): Json<AuthorizeRequest>,
) -> impl IntoResponse {
  authorize_internal(state.database, state.config, authorize_request).await
}

async fn authorize_internal(
  _db: sea_orm::DatabaseConnection,
  config: Arc<AppConfig>,
  authorize_request: AuthorizeRequest,
) -> impl IntoResponse {
  let mut ui_url = match &config.ui_url {
    Some(ui_url) => match Url::parse(&format!("{}/authorize", ui_url)) {
      Ok(ui_url) => ui_url,
      Err(e) => {
        log::error!("invalid config.ui_url: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    },
    None => {
      log::error!("invalid config: missing ui_url");
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  {
    let mut ui_url_params: form_urlencoded::Serializer<'_, url::UrlQuery<'_>> =
      ui_url.query_pairs_mut();

    ui_url_params.append_pair("client_id", &authorize_request.client_id);
    ui_url_params.append_pair("response_type", &authorize_request.response_type.as_str());
    ui_url_params.append_pair("response_mode", &authorize_request.response_mode.as_str());
    ui_url_params.append_pair("scope", &authorize_request.scope);
    ui_url_params.append_pair("redirect_uri", &authorize_request.redirect_uri);
    if let Some(state) = &authorize_request.state {
      ui_url_params.append_pair("state", state);
    }
    if let Some(nonce) = &authorize_request.nonce {
      ui_url_params.append_pair("nonce", nonce);
    }
    if let Some(registration) = &authorize_request.registration {
      ui_url_params.append_pair("registration", registration);
    }
    if let Some(code_challenge) = &authorize_request.code_challenge {
      ui_url_params.append_pair("code_challenge", code_challenge);
    }
    if let Some(code_challenge_method) = &authorize_request.code_challenge_method {
      ui_url_params.append_pair("code_challenge_method", code_challenge_method);
    }
  }

  match Response::builder()
    .status(StatusCode::FOUND)
    .header(header::LOCATION, ui_url.as_str())
    .body(Body::empty())
  {
    Ok(response) => response.into_response(),
    Err(e) => {
      log::error!("Failed to build response: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  post,
  path = "/register-client",
  tags = [TAG],
  request_body(content = ClientRegisterRequest, content_type = "application/json"),
  responses(
    (status = 200, description = "Client registation updated", body = Client),
    (status = 201, description = "Client registered", body = Client),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:create"])
  )
)]
pub async fn register_client(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Json(client_register_request): Json<ClientRegisterRequest>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::ClientWrite) {
    Ok(_) => {}
    Err(e) => {
      log::error!("error registering client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let (client_model, is_new) = match clients::upsert_client(
    &state.database,
    client_register_request.into(),
    crate::core::encryption::random_bytes,
  )
  .await
  {
    Ok(r) => r,
    Err(e) => {
      log::error!("error upserting client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let client: Client = client_model.into();
  if is_new {
    (axum::http::StatusCode::CREATED, axum::Json(client)).into_response()
  } else {
    axum::Json(client).into_response()
  }
}

#[utoipa::path(
  get,
  path = "/user-info",
  tags = [TAG],
  responses(
    (status = 200, description = "Consented claims", body = OpenIdClaims),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn user_info(
  State(_state): State<RouterState>,
  authorization: Authorization<OpenIdClaims>,
) -> impl IntoResponse {
  axum::Json(authorization.claims).into_response()
}

#[utoipa::path(
  post,
  path = "/revoke",
  tags = [TAG],
  request_body(content = RevokeRequest, content_type = "application/x-www-form-urlencoded"),
  responses(
    (status = 204, description = "Token revoked"),
    (status = 400, description = "Invalid request", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn revoke(
  State(state): State<RouterState>,
  Form(revoke_request): Form<RevokeRequest>,
) -> impl IntoResponse {
  let (token_data, _jwk_model) =
    match parse_authorization::<BasicClaims>(&state.database, &state.config, &revoke_request.token)
      .await
    {
      Ok(result) => result,
      Err(e) => {
        log::error!("failed to parse token for revocation: {}", e);
        return StatusCode::OK.into_response();
      }
    };

  if let Some(client_id) = &revoke_request.client_auth.client_id {
    if &token_data.claims.aud != client_id {
      log::error!("client_id mismatch: token belongs to different client");
      return HttpError::unauthorized()
        .with_error("client_id", INVALID_ERROR)
        .into_response();
    }

    match validate_client_authentication(
      &state.database,
      client_id,
      &revoke_request.client_auth,
      GRANT_TYPE_REVOKE,
      &token_data.claims.scope,
    )
    .await
    {
      Ok(_) => {}
      Err(e) => {
        log::error!("client authentication failed: {}", e);
        return e.into_response();
      }
    }
  }

  match revoked_tokens::revoke_token(
    &state.database,
    &revoke_request.token,
    token_data.claims.exp,
  )
  .await
  {
    Ok(_) => {
      log::info!("token successfully revoked");
      StatusCode::NO_CONTENT.into_response()
    }
    Err(e) => {
      log::error!("failed to revoke token: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

#[utoipa::path(
  post,
  path = "/introspect",
  tags = [TAG],
  request_body(content = String, content_type = "application/x-www-form-urlencoded"),
  responses(
    (status = 200, description = "Token introspection result"),
    (status = 400, description = "Invalid request", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn introspect(State(_state): State<RouterState>) -> impl IntoResponse {
  // TODO: Implement RFC 7662 token introspection
  (StatusCode::NOT_IMPLEMENTED, "Not implemented").into_response()
}

#[utoipa::path(
  post,
  path = "/device-authorize",
  tags = [TAG],
  request_body(content = String, content_type = "application/x-www-form-urlencoded"),
  responses(
    (status = 200, description = "Device authorization response"),
    (status = 400, description = "Invalid request", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  )
)]
pub async fn device_authorize(State(_state): State<RouterState>) -> impl IntoResponse {
  // TODO: Implement RFC 8628 device authorization
  (StatusCode::NOT_IMPLEMENTED, "Not implemented").into_response()
}

#[utoipa::path(
  get,
  path = "/client/{client_id}",
  tags = [TAG],
  responses(
    (status = 200, description = "Client fetched", body = Client),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 404, description = "Not Found", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = ["client:read"])
  )
)]
pub async fn client_by_client_id(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(client_id): Path<String>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::ClientRead) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  };

  let client_model = match clients::get_client_by_client_id(&state.database, &client_id).await {
    Ok(Some(client_model)) => client_model,
    Ok(None) => {
      return HttpError::not_found()
        .with_error("client", NOT_FOUND_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("error fetching client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let client: Client = client_model.into();
  axum::Json(client).into_response()
}

#[utoipa::path(
  get,
  path = "/clients/{client_id}/allowed",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = ClientAllowed),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn client_user_allowed(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Path(client_id): Path<String>,
) -> impl IntoResponse {
  match get_user_client_by_client_id(
    &state.database,
    user_authorization.user_model.id,
    &client_id,
  )
  .await
  {
    Ok(Some(user_client_model)) => axum::Json(ClientAllowed {
      allowed_scopes: json_to_string_vec(user_client_model.allowed_scopes),
    })
    .into_response(),
    Ok(None) => HttpError::forbidden()
      .with_error("client", NOT_ALLOWED_ERROR)
      .into_response(),
    Err(e) => {
      log::error!("error fetching user client: {}", e);
      HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response()
    }
  }
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(jwks))
    .routes(routes!(openid_configuration))
    .routes(routes!(end_session))
    .routes(routes!(token))
    .routes(routes!(user_info))
    .routes(routes!(post_authorize))
    .routes(routes!(authorize))
    .routes(routes!(register_client))
    .routes(routes!(revoke))
    .routes(routes!(introspect))
    .routes(routes!(device_authorize))
    .routes(routes!(client_by_client_id))
    .routes(routes!(client_user_allowed))
    .with_state(state)
}
