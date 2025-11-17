use std::collections::HashSet;

use axum::{
  extract::{Form, State},
  response::IntoResponse,
};
use http::StatusCode;
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
    client::sql::{ClientSQLRow, get_client_by_client_id, upsert_client},
    user::sql::{
      get_user_active_password_by_user_id, get_user_by_id, get_user_by_username_or_primary_email,
      get_user_client_by_client_id,
    },
  },
  router::{
    client::constants::CLIENT_CREATE,
    common::{
      constants::{
        TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE, TOKEN_ISSUE_TYPE_PASSWORD,
        TOKEN_ISSUE_TYPE_REFRESH_TOKEN, TOKEN_TYPE_AUTHORIZATION_CODE, TOKEN_TYPE_REFRESH,
      },
      entity::{BasicClaims, Token},
      helper::{create_user_auhorization_code_token, create_user_token},
    },
    entity::RouterState,
    error::{
      CREDENTIALS, HttpError, INTERNAL_ERROR, INVALID_ERROR, NOT_ALLOWED_ERROR, NOT_FOUND_ERROR,
    },
    json::Json,
    middleware::{authorization::parse_authorization, user_authorization::UserAuthorization},
    oidc::{
      constants::TAG,
      entity::{
        Authorization, AuthorizationRequest, Client, ClientRegisterRequest, JWK, JWKs,
        OpenIdConfiguration, TokenRequest, TokenRequestCommon,
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

  axum::Json(OpenIdConfiguration {
    authorization_endpoint: if let Some(ui_url) = &state.config.ui_url {
      Some(format!("{}/authorize", ui_url))
    } else {
      None
    },
    device_authorization_endpoint: if let Some(ui_url) = &state.config.ui_url {
      Some(format!("{}/device-authorize", ui_url))
    } else {
      None
    },
    token_endpoint: format!("{}/token", api_url),
    userinfo_endpoint: Some(format!("{}/current-user", api_url)),
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
    response_modes_supported: vec![
      "query".to_owned(),
      "fragment".to_owned(),
      "form_post".to_owned(),
    ],
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
      common,
      username,
      password,
    } => password_grant(state, common, username, password).await,
    TokenRequest::RefreshToken {
      common,
      refresh_token,
    } => refresh_token_grant(state, common, refresh_token).await,
    TokenRequest::AuthorizationCode { common, code } => {
      authorization_code_grant(state, common, code).await
    }
  };
  match token_result {
    Ok(token) => axum::Json(token).into_response(),
    Err(e) => e.into_response(),
  }
}

async fn password_grant(
  state: RouterState,
  common: TokenRequestCommon,
  username: String,
  password: String,
) -> Result<Token, HttpError> {
  let audiences = get_audiences_by_client_id(
    &state.pool,
    &state.config,
    common.client_id.as_ref().map(AsRef::as_ref),
  )
  .await?;

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
    common.scope.or_else(|| Some("openid".to_owned())),
    TOKEN_ISSUE_TYPE_PASSWORD.to_owned(),
    &audiences,
  )
  .await
}

async fn refresh_token_grant(
  state: RouterState,
  common: TokenRequestCommon,
  refresh_token: String,
) -> Result<Token, HttpError> {
  let audiences = get_audiences_by_client_id(
    &state.pool,
    &state.config,
    common.client_id.as_ref().map(AsRef::as_ref),
  )
  .await?;

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
    Some(token_data.claims.scopes.join(" ").to_owned()),
    TOKEN_ISSUE_TYPE_REFRESH_TOKEN.to_owned(),
    &audiences,
  )
  .await
}

async fn authorization_code_grant(
  state: RouterState,
  common: TokenRequestCommon,
  code: String,
) -> Result<Token, HttpError> {
  let audiences = get_audiences_by_client_id(
    &state.pool,
    &state.config,
    common.client_id.as_ref().map(AsRef::as_ref),
  )
  .await?;

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
    Some(token_data.claims.scopes.join(" ").to_owned()),
    TOKEN_ISSUE_TYPE_AUTHORIZATION_CODE.to_owned(),
    &audiences,
  )
  .await
}

fn get_audiences_by_client(client_sql_row: &ClientSQLRow) -> Result<Vec<String>, HttpError> {
  let audiences = client_sql_row
    .audience
    .as_ref()
    .map(json_to_string_vec)
    .unwrap_or_default();

  if audiences.is_empty() {
    return Err(HttpError::unauthorized().with_error("client", NOT_FOUND_ERROR));
  }
  Ok(audiences)
}

async fn get_audiences_by_client_id(
  pool: &sqlx::AnyPool,
  config: &AppConfig,
  client_id: Option<&str>,
) -> Result<Vec<String>, HttpError> {
  if let Some(client_id) = client_id {
    match get_client_by_client_id(pool, client_id).await {
      Ok(Some(client_sql_row)) => get_audiences_by_client(&client_sql_row),
      Ok(None) => return Err(HttpError::unauthorized().with_error("client_uri", INVALID_ERROR)),
      Err(e) => {
        log::error!("error fetching client: {}", e);
        return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
      }
    }
  } else {
    Ok(vec![config.api_url()])
  }
}

#[utoipa::path(
  post,
  path = "/authorize",
  tags = [TAG],
  request_body(content = AuthorizationRequest, content_type = "application/x-www-form-urlencoded"),
  responses(
    (status = 200, description = "Authorized", body = Authorization),
    (status = 400, description = "Application Error", body = HttpError),
    (status = 401, description = "Application Error", body = HttpError),
    (status = 403, description = "Application Error", body = HttpError),
    (status = 500, description = "Application Error", body = HttpError),
  ),
  security(
    ("Authorization" = [])
  )
)]
pub async fn authorize(
  State(state): State<RouterState>,
  user_authorization: UserAuthorization,
  Form(authorization_request): Form<AuthorizationRequest>,
) -> impl IntoResponse {
  let client_sql_row =
    match get_client_by_client_id(&state.pool, &authorization_request.client_id).await {
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

  match get_user_client_by_client_id(
    &state.pool,
    user_authorization.user_sql_row.id,
    &client_sql_row.client_id,
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

  if let Some(redirect_uris) = client_sql_row
    .redirect_uris
    .as_ref()
    .map(json_to_string_vec)
  {
    let redirect_uri_string = redirect_uri.origin().ascii_serialization() + redirect_uri.path();
    log::info!("{:?} ~ {:?}", redirect_uris, redirect_uri_string);
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

  let audiences = match get_audiences_by_client(&client_sql_row) {
    Ok(audiences) => audiences,
    Err(e) => return e.into_response(),
  };

  let authorization_response = match authorization_request.response_type {
    super::entity::ResponseType::Code => match create_user_auhorization_code_token(
      &state.pool,
      &state.config,
      user_authorization.user_sql_row,
      &audiences,
    )
    .await
    {
      Ok(code) => Authorization::AuthorizationCode { code },
      Err(e) => {
        return e.into_response();
      }
    },
    super::entity::ResponseType::IdToken => todo!(),
    super::entity::ResponseType::IdTokenToken => todo!(),
    super::entity::ResponseType::CodeIdTokenToken => todo!(),
    super::entity::ResponseType::CodeToken => todo!(),
    super::entity::ResponseType::None => todo!(),
  };

  axum::Json(authorization_response).into_response()
}

#[utoipa::path(
  post,
  path = "/register-client",
  tags = [TAG],
  request_body(content = ClientRegisterRequest, content_type = "application/json; charset=utf-8"),
  responses(
    (status = 200, description = "Client registation updated", body = Client),
    (status = 201, description = "Client registered", body = Client),
    (status = 401, description = "Application Error", body = HttpError),
    (status = 403, description = "Application Error", body = HttpError),
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
  match user_authorization.has_permission(CLIENT_CREATE) {
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

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(jwks))
    .routes(routes!(openid_configuration))
    .routes(routes!(token))
    .routes(routes!(authorize))
    .routes(routes!(register_client))
    .with_state(state)
}
