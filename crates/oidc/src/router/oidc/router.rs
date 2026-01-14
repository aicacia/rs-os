use std::{collections::HashSet, sync::Arc};

use axum::{
  body::Body,
  extract::{Query, State},
  response::{IntoResponse, Response},
};
use base64::Engine;
use http::{StatusCode, header};
use os_api::{
  Form, Json,
  error::{
    CREDENTIALS, HttpError, INTERNAL_ERROR, INVALID_ERROR, NOT_ALLOWED_ERROR, NOT_FOUND_ERROR,
    NOT_SUPPORTED_ERROR, REQUIRED_ERROR,
  },
};
use os_oidc_model::entities::{
  clients,
  jwks::{get_jwk_for_sign_and_verify, list_jwks},
  revoked_tokens,
  users::{self, get_user_client_by_client_id},
};
use sha2::{Digest, Sha256};
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  config::AppConfig,
  core::{encryption::verify_password, helper::json_to_string_vec},
  router::{
    common::{
      constants::{
        TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE, TOKEN_ISSUE_TYPE_PASSWORD,
        TOKEN_ISSUE_TYPE_REFRESH_TOKEN, TOKEN_TYPE_AUTHORIZATION_CODE, TOKEN_TYPE_REFRESH,
      },
      entity::{AuthorizationCodeClaims, BasicClaims, Permission, Token, UserInfo},
      helper::{create_user_authorization_code_token, create_user_token},
    },
    entity::RouterState,
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
        ApproveClientQuery, AuthorizeRequest, Client, ClientAllowed, ClientAllowedQuery,
        ClientAuthentication, ClientAuthorization, ClientAuthorizeRequest, ClientByClientIdQuery,
        ClientRegisterRequest, EndSessionRequest, JWK, JWKs, OpenIdConfiguration, RevokeRequest,
        TokenRequest,
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
  let jwks = match list_jwks(&state.database_connection).await {
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
  let api_url = state.config.url();

  let jwks = match list_jwks(&state.database_connection).await {
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

    if key_operations.iter().any(|s| s.as_str() == "sign") {
      signing_algs.insert(jwk.alg.to_owned());
    }
  }

  let (grant_types_list, response_types_list, scopes_list, auth_methods_list) = match tokio::try_join!(
    clients::get_distinct_grant_types(&state.database_connection),
    clients::get_distinct_response_types(&state.database_connection),
    clients::get_distinct_scopes(&state.database_connection),
    clients::get_distinct_auth_methods(&state.database_connection),
  ) {
    Ok(result) => result,
    Err(e) => {
      log::error!("error getting client configuration: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  };

  let mut grant_types_supported = HashSet::new();
  for grant_types_json in grant_types_list {
    let grant_types: Vec<String> = json_to_string_vec(&grant_types_json);
    for grant_type in grant_types {
      grant_types_supported.insert(grant_type);
    }
  }

  let mut response_types_supported = HashSet::new();
  for response_types_json in response_types_list {
    let response_types: Vec<String> = json_to_string_vec(&response_types_json);
    for response_type in response_types {
      response_types_supported.insert(response_type);
    }
  }

  let mut scopes_supported = HashSet::new();
  for scopes_json in scopes_list {
    let scopes: Vec<String> = json_to_string_vec(&scopes_json);
    for scope in scopes {
      scopes_supported.insert(scope);
    }
  }

  let token_endpoint_auth_methods_supported: HashSet<String> =
    auth_methods_list.into_iter().collect();

  let mut response_modes_supported = HashSet::from([
    "query".to_owned(),
    "fragment".to_owned(),
    "form_post".to_owned(),
  ]);

  if state.config.ui_url.is_some() {
    response_modes_supported.insert("web_message".to_owned());
  }

  let claims_supported = HashSet::from([
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
  ]);

  let code_challenge_methods_supported = HashSet::from(["S256".to_owned()]);

  let subject_types_supported = HashSet::from(["public".to_owned(), "pairwise".to_owned()]);

  axum::Json(OpenIdConfiguration {
    authorization_endpoint: format!("{}/authorize", api_url),
    device_authorization_endpoint: format!("{}/device-authorize", api_url),
    token_endpoint: format!("{}/token", api_url),
    end_session_endpoint: Some(format!("{}/end-session", api_url)),
    userinfo_endpoint: Some(format!("{}/user-info", api_url)),
    revocation_endpoint: Some(format!("{}/revoke", api_url)),
    registration_endpoint: Some(format!("{}/register-client", api_url)),
    jwks_uri: format!("{}/.well-known/jwks.json", api_url),
    response_types_supported: response_types_supported.into_iter().collect(),
    response_modes_supported: response_modes_supported.into_iter().collect(),
    subject_types_supported: subject_types_supported.into_iter().collect(),
    id_token_signing_alg_values_supported: signing_algs.into_iter().collect(),
    scopes_supported: scopes_supported.into_iter().collect(),
    token_endpoint_auth_methods_supported: token_endpoint_auth_methods_supported
      .into_iter()
      .collect(),
    claims_supported: claims_supported.into_iter().collect(),
    code_challenge_methods_supported: code_challenge_methods_supported.into_iter().collect(),
    grant_types_supported: grant_types_supported.into_iter().collect(),
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
        match parse_authorization::<BasicClaims>(
          &state.database_connection,
          &state.config,
          &id_token_hint,
        )
        .await
        {
          Ok((token, _jwk)) => token.claims.client,
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

  let client_model =
    match clients::get_client_by_client_id(&state.database_connection, &client_id).await {
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
  let client_model = validate_client_authentication(
    &state.database_connection,
    &client_auth,
    GRANT_TYPE_PASSWORD,
    &scope,
  )
  .await?;

  let user =
    match users::get_user_by_username_or_primary_email(&state.database_connection, &username).await
    {
      Ok(Some(user)) => user,
      Ok(None) => return Err(HttpError::unauthorized().with_error(CREDENTIALS, INVALID_ERROR)),
      Err(e) => {
        log::error!("error getting user: {}", e);
        return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
      }
    };

  let user_password =
    match users::get_user_active_password_by_user_id(&state.database_connection, user.id).await {
      Ok(Some(user_password)) => user_password,
      Ok(None) => {
        return Err(HttpError::forbidden().with_error(CREDENTIALS, TOKEN_ISSUE_TYPE_PASSWORD));
      }
      Err(e) => {
        log::error!(
          "error fetching user password from database_connection: {}",
          e
        );
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

  let jwk = match get_jwk_for_sign_and_verify(&state.database_connection).await {
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
    &state.database_connection,
    &state.config,
    jwk,
    user,
    client_model.client_id,
    json_to_string_vec(client_model.audience),
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
    parse_authorization::<BasicClaims>(&state.database_connection, &state.config, &refresh_token)
      .await?;

  if token_data.claims.r#type != TOKEN_TYPE_REFRESH {
    return Err(HttpError::unauthorized());
  }
  if token_data.claims.client != client_auth.client_id {
    return Err(HttpError::unauthorized().with_error("client_id", INVALID_ERROR));
  }

  let client_model = validate_client_authentication(
    &state.database_connection,
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

  let user = match users::get_user_by_id(&state.database_connection, user_id).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized()),
    Err(e) => {
      log::error!("error fetching user: {}", e);
      return Err(HttpError::internal_error());
    }
  };

  create_user_token(
    &state.database_connection,
    &state.config,
    jwk_model,
    user,
    client_model.client_id,
    json_to_string_vec(client_model.audience),
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
  let (token_data, jwk_model) = parse_authorization::<AuthorizationCodeClaims>(
    &state.database_connection,
    &state.config,
    &code,
  )
  .await?;

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

  let client_model = validate_client_authentication(
    &state.database_connection,
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

  let user = match users::get_user_by_id(&state.database_connection, user_id).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized()),
    Err(e) => {
      log::error!("error fetching user: {}", e);
      return Err(HttpError::internal_error());
    }
  };

  create_user_token(
    &state.database_connection,
    &state.config,
    jwk_model,
    user,
    client_model.client_id,
    json_to_string_vec(client_model.audience),
    token_data.claims.basic_claims.scope,
    TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE.to_owned(),
  )
  .await
}

pub(crate) async fn validate_client_authentication(
  db: &sea_orm::DatabaseConnection,
  client_auth: &ClientAuthentication,
  grant_type: &str,
  scope: &str,
) -> Result<clients::Model, HttpError> {
  let client_model =
    match clients::get_client_by_client_id(db, client_auth.client_id.as_str()).await {
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
    "client_secret_post" | "client_secret_basic" => match &client_auth.client_secret {
      Some(provided_secret) => {
        if provided_secret != &client_model.client_secret {
          return Err(HttpError::unauthorized().with_error("client_secret", INVALID_ERROR));
        }
      }
      None => {
        return Err(HttpError::bad_request().with_error("client_secret", REQUIRED_ERROR));
      }
    },
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
  authorize_internal(state.database_connection, state.config, authorize_request).await
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
  authorize_internal(state.database_connection, state.config, authorize_request).await
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
    ui_url_params.append_pair("response_type", authorize_request.response_type.as_str());
    ui_url_params.append_pair("response_mode", authorize_request.response_mode.as_str());
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

  let client_active_model: os_oidc_model::entities::clients::ActiveModel =
    client_register_request.into();

  let (client_model, is_new) = match clients::upsert_client(
    &state.database_connection,
    client_active_model,
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
    (status = 200, description = "Consented claims", body = UserInfo),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn user_info(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
) -> impl IntoResponse {
  match user_authorization
    .get_user_info(&state.database_connection)
    .await
  {
    Ok(user_info) => axum::Json(user_info).into_response(),
    Err(e) => e.into_response(),
  }
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
  let (token_data, _jwk_model) = match parse_authorization::<BasicClaims>(
    &state.database_connection,
    &state.config,
    &revoke_request.token,
  )
  .await
  {
    Ok(result) => result,
    Err(e) => {
      log::error!("failed to parse token for revocation: {}", e);
      return StatusCode::OK.into_response();
    }
  };

  if token_data.claims.client != revoke_request.client_auth.client_id {
    log::error!("client_id mismatch: token belongs to different client");
    return HttpError::unauthorized()
      .with_error("client_id", INVALID_ERROR)
      .into_response();
  }

  match validate_client_authentication(
    &state.database_connection,
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

  match revoked_tokens::revoke_token(
    &state.database_connection,
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
    (status = 200, description = "Token introspection result", body = BasicClaims),
    (status = 400, description = "Invalid request", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn introspect(
  State(_state): State<RouterState>,
  Authorization { claims }: Authorization<BasicClaims>,
) -> impl IntoResponse {
  axum::Json(claims).into_response()
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
  path = "/client",
  tags = [TAG],
  params(ClientByClientIdQuery),
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
pub async fn client(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Query(ClientByClientIdQuery { client_id }): Query<ClientByClientIdQuery>,
) -> impl IntoResponse {
  match user_authorization.has_permission(Permission::ClientRead) {
    Ok(_) => {}
    Err(e) => return e.into_response(),
  };

  let client_model =
    match clients::get_client_by_client_id(&state.database_connection, &client_id).await {
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
  post,
  path = "/authorize-client",
  tags = [TAG],
  request_body(content = ClientAuthorizeRequest, content_type = "application/json"),
  responses(
    (status = 200, description = "Authorized", body = ClientAuthorization),
    (status = 400, description = "Application Error", body = HttpError),
    (status = 401, description = "Unauthorized", body = HttpError),
    (status = 403, description = "Forbidden", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn authorize_client(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Json(authorization_request): Json<ClientAuthorizeRequest>,
) -> impl IntoResponse {
  let client_model = match clients::get_client_by_client_id(
    &state.database_connection,
    &authorization_request.client_id,
  )
  .await
  {
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

  match get_user_client_by_client_id(
    &state.database_connection,
    user_authorization.user_model.id,
    &authorization_request.client_id,
  )
  .await
  {
    Ok(Some(_client_allowed)) => {}
    Ok(None) => {
      return HttpError::forbidden()
        .with_error("client", NOT_ALLOWED_ERROR)
        .into_response();
    }
    Err(e) => {
      log::error!("failed to check user client: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  }

  let redirect_uri = match Url::parse(&authorization_request.redirect_uri) {
    Ok(redirect_uri) => redirect_uri,
    Err(e) => {
      log::error!("invalid redirect_uri: {}", e);
      return HttpError::bad_request()
        .with_error("redirect_uri", INVALID_ERROR)
        .into_response();
    }
  };

  if let Some(redirect_uris) = client_model.redirect_uris.as_ref().map(json_to_string_vec) {
    let redirect_uri_string = redirect_uri.origin().ascii_serialization() + redirect_uri.path();
    if !redirect_uris.contains(&redirect_uri_string) {
      return HttpError::bad_request()
        .with_error("redirect_uri", NOT_ALLOWED_ERROR)
        .into_response();
    }
  } else {
    return HttpError::bad_request()
      .with_error("client", INVALID_ERROR)
      .into_response();
  }

  let authorization_code_token = if authorization_request.response_type.needs_code() {
    let (code_challenge, code_challenge_method) = match (
      &authorization_request.code_challenge,
      &authorization_request.code_challenge_method,
    ) {
      (Some(challenge), Some(method)) => (challenge.clone(), method.clone()),
      _ => {
        return HttpError::bad_request()
          .with_error("code_challenge", INVALID_ERROR)
          .into_response();
      }
    };

    match create_user_authorization_code_token(
      &state.database_connection,
      &state.config,
      user_authorization.user_model.id,
      client_model.client_id,
      json_to_string_vec(client_model.audience),
      authorization_request.scope,
      code_challenge,
      code_challenge_method,
    )
    .await
    {
      Ok(code) => Some(code),
      Err(e) => {
        return e.into_response();
      }
    }
  } else {
    None
  };

  if let Some(authorization_code_token) = authorization_code_token {
    axum::Json(ClientAuthorization::AuthorizationCode {
      code: authorization_code_token,
    })
    .into_response()
  } else {
    HttpError::bad_request()
      .with_error("response_type", INVALID_ERROR)
      .into_response()
  }
}

#[utoipa::path(
  get,
  path = "/client-allowed",
  tags = [TAG],
  params(ClientAllowedQuery),
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
pub async fn is_client_allowed_for_user(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Query(ClientAllowedQuery { client_id }): Query<ClientAllowedQuery>,
) -> impl IntoResponse {
  match get_user_client_by_client_id(
    &state.database_connection,
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

#[utoipa::path(
  post,
  path = "/approve-client",
  tags = [TAG],
  params(ApproveClientQuery),
  responses(
    (status = 204, content_type = "application/json"),
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
pub async fn approve_client_for_user(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Query(ApproveClientQuery { client_id }): Query<ApproveClientQuery>,
) -> impl IntoResponse {
  let client_model =
    match clients::get_client_by_client_id(&state.database_connection, &client_id).await {
      Ok(Some(client)) => client,
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

  let scopes = json_to_string_vec(&client_model.scopes);
  match users::upsert_user_client(
    &state.database_connection,
    user_authorization.user_model.id,
    &client_id,
    scopes,
  )
  .await
  {
    Ok(_user_client) => {}
    Err(e) => {
      log::error!("error approving client for user: {}", e);
      return HttpError::internal_error()
        .with_application_error(INTERNAL_ERROR)
        .into_response();
    }
  }

  axum::Json(()).into_response()
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
    .routes(routes!(client))
    .routes(routes!(authorize_client))
    .routes(routes!(is_client_allowed_for_user))
    .routes(routes!(approve_client_for_user))
    .with_state(state)
}
