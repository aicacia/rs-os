use std::{collections::HashSet, sync::Arc};

use axum::{
  body::Body,
  extract::{Query, State},
  response::{IntoResponse, Response},
};
use http::{StatusCode, header};
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
  core::{
    config::app_config::AppConfig,
    encryption::verify_password,
    helper::json_to_string_vec,
    jwk::sql::{get_jwk_for_sign_and_verify, list_jwks},
  },
  model::{
    client::sql::{get_client_by_client_id, upsert_client},
    user::sql::{
      get_user_active_password_by_user_id, get_user_by_id, get_user_by_username_or_primary_email,
    },
  },
  router::{
    common::permissions::Permission,
    common::{
      constants::{
        TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE, TOKEN_ISSUE_TYPE_PASSWORD,
        TOKEN_ISSUE_TYPE_REFRESH_TOKEN, TOKEN_TYPE_AUTHORIZATION_CODE, TOKEN_TYPE_REFRESH,
      },
      entity::{BasicClaims, OpenIdClaims, Token},
      helper::create_user_token,
    },
    entity::RouterState,
    error::{
      CREDENTIALS, HttpError, INTERNAL_ERROR, INVALID_ERROR, NOT_ALLOWED_ERROR, NOT_FOUND_ERROR,
    },
    form::Form,
    json::Json,
    middleware::{
      authorization::{Authorization, parse_authorization},
      user_authorization::UserAuthorization,
    },
    oidc::{
      constants::TAG,
      entity::{
        AuthorizeRequest, Client, ClientRegisterRequest, EndSessionRequest, JWK, JWKs,
        OpenIdConfiguration, TokenRequest,
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
  let jwks = match list_jwks(&state.pool).await {
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

  let jwks = match list_jwks(&state.pool).await {
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
    response_types_supported: vec![
      "code".to_owned(),
      "token".to_owned(),
      "id_token".to_owned(),
      "code token".to_owned(),
      "code id_token".to_owned(),
      "token id_token".to_owned(),
      "code token id_token".to_owned(),
      "none".to_owned(),
    ],
    response_modes_supported: response_modes_supported,
    subject_types_supported: vec!["public".to_owned(), "pairwise".to_owned()],
    id_token_signing_alg_values_supported: signing_algs.into_iter().collect(),
    scopes_supported: vec![
      "openid".to_owned(),
      "profile".to_owned(),
      "address".to_owned(),
      "offline".to_owned(),
      "email".to_owned(),
      "phone_number".to_owned(),
    ],
    token_endpoint_auth_methods_supported: vec![
      "client_secret_post".to_owned(),
      "client_secret_basic".to_owned(),
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
      "phone_number".to_owned(),
      "phone_number_verified".to_owned(),
    ],
    code_challenge_methods_supported: vec!["plain".to_owned(), "S256".to_owned()],
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
        match parse_authorization::<BasicClaims>(&state.pool, &state.config, &id_token_hint).await {
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

  let client_sql_row = match get_client_by_client_id(&state.pool, &client_id).await {
    Ok(Some(client_sql_row)) => client_sql_row,
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

  let post_logout_redirect_uri_string = if let Some(post_logout_redirect_uris) = client_sql_row
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
    } => password_grant(state, scope, username, password).await,
    TokenRequest::RefreshToken { refresh_token } => refresh_token_grant(state, refresh_token).await,
    TokenRequest::AuthorizationCode { code } => authorization_code_grant(state, code).await,
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
) -> Result<Token, HttpError> {
  let user = match get_user_by_username_or_primary_email(&state.pool, &username).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized().with_error(CREDENTIALS, INVALID_ERROR)),
    Err(e) => {
      log::error!("error getting user: {}", e);
      return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
    }
  };

  let user_password = match get_user_active_password_by_user_id(&state.pool, user.id).await {
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
      return Err(HttpError::unauthorized().with_error(CREDENTIALS, INVALID_ERROR));
    }
    Err(e) => {
      log::error!("error verifying user password: {}", e);
      return Err(HttpError::unauthorized().with_application_error(INTERNAL_ERROR));
    }
  }

  let jwk = match get_jwk_for_sign_and_verify(&state.pool).await {
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
    &state.pool,
    &state.config,
    jwk,
    user,
    state.config.api_url(),
    scope,
    TOKEN_ISSUE_TYPE_PASSWORD.to_owned(),
  )
  .await
}

async fn refresh_token_grant(
  state: RouterState,
  refresh_token: String,
) -> Result<Token, HttpError> {
  let (token_data, jwk_sql_row) =
    parse_authorization::<BasicClaims>(&state.pool, &state.config, &refresh_token).await?;

  if token_data.claims.r#type != TOKEN_TYPE_REFRESH {
    return Err(HttpError::unauthorized());
  }

  let user = match get_user_by_id(&state.pool, token_data.claims.sub).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized()),
    Err(e) => {
      log::error!("error fetching user: {}", e);
      return Err(HttpError::internal_error());
    }
  };

  create_user_token(
    &state.pool,
    &state.config,
    jwk_sql_row,
    user,
    token_data.claims.aud,
    token_data.claims.scope,
    TOKEN_ISSUE_TYPE_REFRESH_TOKEN.to_owned(),
  )
  .await
}

async fn authorization_code_grant(state: RouterState, code: String) -> Result<Token, HttpError> {
  let (token_data, jwk_sql_row) =
    parse_authorization::<BasicClaims>(&state.pool, &state.config, &code).await?;

  if token_data.claims.r#type != TOKEN_TYPE_AUTHORIZATION_CODE {
    return Err(HttpError::unauthorized());
  }

  let user = match get_user_by_id(&state.pool, token_data.claims.sub).await {
    Ok(Some(user)) => user,
    Ok(None) => return Err(HttpError::unauthorized()),
    Err(e) => {
      log::error!("error fetching user: {}", e);
      return Err(HttpError::internal_error());
    }
  };

  create_user_token(
    &state.pool,
    &state.config,
    jwk_sql_row,
    user,
    token_data.claims.aud,
    token_data.claims.scope,
    TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE.to_owned(),
  )
  .await
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
  authorize_internal(state.pool, state.config, authorize_request).await
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
  authorize_internal(state.pool, state.config, authorize_request).await
}

async fn authorize_internal(
  pool: sqlx::AnyPool,
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

  let client_sql_row = match get_client_by_client_id(&pool, &authorize_request.client_id).await {
    Ok(Some(client_sql_row)) => client_sql_row,
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

  let redirect_uri = match Url::parse(&authorize_request.redirect_uri) {
    Ok(redirect_uri) => redirect_uri,
    Err(e) => {
      log::error!("invalid redirect_uri: {}", e);
      return HttpError::bad_request()
        .with_error("redirect_uri", INVALID_ERROR)
        .into_response();
    }
  };

  if let Some(redirect_uris) = client_sql_row
    .redirect_uris
    .as_ref()
    .map(json_to_string_vec)
  {
    let redirect_uri_string = redirect_uri.origin().ascii_serialization() + redirect_uri.path();
    if !redirect_uris.contains(&redirect_uri_string) {
      return HttpError::bad_request()
        .with_error("redirect_uri", NOT_ALLOWED_ERROR)
        .into_response();
    }
    redirect_uri_string
  } else {
    return HttpError::bad_request()
      .with_error("client", INVALID_ERROR)
      .into_response();
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

  let (client_sql_row, is_new) =
    match upsert_client(&state.pool, client_register_request.into()).await {
      Ok(result) => result,
      Err(e) => {
        log::error!("error registering client: {}", e);
        return HttpError::internal_error()
          .with_application_error(INTERNAL_ERROR)
          .into_response();
      }
    };

  let client: Client = client_sql_row.into();

  (
    if is_new {
      StatusCode::CREATED
    } else {
      StatusCode::OK
    },
    axum::Json(client),
  )
    .into_response()
}

#[utoipa::path(
  get,
  path = "/user-info",
  tags = [TAG],
  request_body(content = ClientRegisterRequest, content_type = "application/json"),
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
    .with_state(state)
}
