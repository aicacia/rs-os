#[derive(sqlx::FromRow)]
pub struct ClientSQLRow {
  pub id: i64,
  pub active: i64,
  pub name: String,
  pub client_id: String,
  pub client_secret: String,
  pub redirect_uris: Option<String>,
  pub post_logout_redirect_uris: Option<String>,
  pub logo_uri: Option<String>,
  pub client_uri: Option<String>,
  pub policy_uri: Option<String>,
  pub terms_of_service_uri: Option<String>,
  pub application_type: String,
  pub auth_method: String,
  pub grant_types: String,
  pub response_types: String,
  pub scopes: String,
  pub audience: Option<String>,
  pub access_token_expires_in_seconds: i64,
  pub id_token_expires_in_seconds: i64,
  pub refresh_expires_in_seconds: i64,
  pub updated_at: i64,
  pub created_at: i64,
}

impl ClientSQLRow {
  pub fn is_active(&self) -> bool {
    self.active != 0
  }
}

pub async fn get_client_by_client_id(
  pool: &sqlx::AnyPool,
  client_id: &str,
) -> sqlx::Result<Option<ClientSQLRow>> {
  sqlx::query_as(
    r#"SELECT c.*
    FROM clients c
    WHERE c.client_id = $1
    LIMIT 1;"#,
  )
  .bind(client_id)
  .fetch_optional(pool)
  .await
}
