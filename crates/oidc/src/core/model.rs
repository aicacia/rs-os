use std::io;

use jsonwebtoken::Algorithm;
use os_oidc_model::entities::clients::{self, ActiveModel};
use os_oidc_model::entities::jwks::{create_jwk, generate_jwk, list_jwks};
use sea_orm::DatabaseConnection;
use sea_orm::Set;

use crate::config::AppConfig;
use crate::core::encryption::random_bytes;
use crate::core::helper::string_vec_to_json;

pub async fn ensure_jwk_exists(db: &DatabaseConnection, default_alg: Algorithm) -> io::Result<()> {
  let has_any = !list_jwks(db).await.map_err(io::Error::other)?.is_empty();
  if !has_any {
    let jwk = generate_jwk(default_alg).map_err(io::Error::other)?;
    let _ = create_jwk(db, jwk).await.map_err(io::Error::other)?;
  }
  Ok(())
}

pub async fn ensure_oidc_client_exists(
  db: &DatabaseConnection,
  config: &AppConfig,
) -> io::Result<()> {
  let client_id = config.ui_url();

  if let Some(_existing) = clients::get_client_by_client_id(db, &client_id)
    .await
    .map_err(io::Error::other)?
  {
    return Ok(());
  }

  let now = chrono::Utc::now().timestamp();

  let mut model: ActiveModel = ActiveModel {
    name: Set("OIDC UI".to_string()),
    client_id: Set(client_id.clone()),
    auth_method: Set("none".to_string()),
    application_type: Set("web".to_string()),
    grant_types: Set(string_vec_to_json(&vec![
      "password".to_string(),
      "authorization_code".to_string(),
      "refresh_token".to_string(),
    ])),
    response_types: Set(string_vec_to_json(&vec![
      "code".to_string(),
      "none".to_string(),
    ])),
    scopes: Set(string_vec_to_json(&vec![
      "openid".to_string(),
      "profile".to_string(),
      "email".to_string(),
      "address".to_string(),
      "phone".to_string(),
      "offline".to_string(),
    ])),
    audience: Set(string_vec_to_json(&vec![client_id.clone()])),
    access_token_expires_in_seconds: Set(config.token.expires_in_seconds as i64),
    id_token_expires_in_seconds: Set(config.token.expires_in_seconds as i64),
    refresh_expires_in_seconds: Set(config.token.refresh_expires_in_seconds as i64),
    active: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };

  if let Some(ui_url) = &config.ui_url {
    model.redirect_uris = Set(Some(string_vec_to_json(&vec![ui_url.clone()])));
    model.post_logout_redirect_uris = Set(Some(string_vec_to_json(&vec![ui_url.clone()])));
  }

  let _ = clients::upsert_client(db, model, random_bytes)
    .await
    .map_err(io::Error::other)?;

  Ok(())
}

pub async fn init(db: &DatabaseConnection, config: &AppConfig) -> io::Result<()> {
  ensure_jwk_exists(db, config.token.default_jwt_algorithm).await?;
  ensure_oidc_client_exists(db, config).await?;
  Ok(())
}
