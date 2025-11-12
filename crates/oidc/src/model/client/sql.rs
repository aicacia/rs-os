use crate::core::encryption::random_bytes;

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

#[derive(sqlx::FromRow)]
pub struct ClientSQLUpsert {
  pub name: String,
  pub client_id: String,
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
}

pub async fn upsert_client(
  pool: &sqlx::AnyPool,
  client: ClientSQLUpsert,
) -> sqlx::Result<ClientSQLRow> {
  sqlx::query_as(
    r#"INSERT INTO clients (
            name,
            client_id,
            client_secret,
            redirect_uris,
            post_logout_redirect_uris,
            logo_uri,
            client_uri,
            policy_uri,
            terms_of_service_uri,
            application_type,
            auth_method,
            grant_types,
            response_types,
            scopes,
            audience,
            access_token_expires_in_seconds,
            id_token_expires_in_seconds,
            refresh_expires_in_seconds
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8,
            $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
        )
        ON CONFLICT(client_id) DO UPDATE SET
            name = $1,
            redirect_uris = $4,
            post_logout_redirect_uris = $5,
            logo_uri = $6,
            client_uri = $7,
            policy_uri = $8,
            terms_of_service_uri = $9,
            application_type = $10,
            auth_method = $11,
            grant_types = $12,
            response_types = $13,
            scopes = $14,
            audience = $15,
            access_token_expires_in_seconds = $16,
            id_token_expires_in_seconds = $17,
            refresh_expires_in_seconds = $18
        RETURNING *;"#,
  )
  .bind(client.name)
  .bind(client.client_id)
  .bind(hex::encode(random_bytes(256)))
  .bind(client.redirect_uris)
  .bind(client.post_logout_redirect_uris)
  .bind(client.logo_uri)
  .bind(client.client_uri)
  .bind(client.policy_uri)
  .bind(client.terms_of_service_uri)
  .bind(client.application_type)
  .bind(client.auth_method)
  .bind(client.grant_types)
  .bind(client.response_types)
  .bind(client.scopes)
  .bind(client.audience)
  .bind(client.access_token_expires_in_seconds)
  .bind(client.id_token_expires_in_seconds)
  .bind(client.refresh_expires_in_seconds)
  .fetch_one(pool)
  .await
}
