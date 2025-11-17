use os_db::pool::run_transaction;

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
  get_client_by_client_id_internal(pool, client_id).await
}

async fn get_client_by_client_id_internal<'e, E>(
  executor: E,
  client_id: &str,
) -> sqlx::Result<Option<ClientSQLRow>>
where
  E: 'e + sqlx::Executor<'e, Database = sqlx::Any>,
{
  sqlx::query_as(
    r#"SELECT c.*
    FROM clients c
    WHERE c.client_id = $1
    LIMIT 1;"#,
  )
  .bind(client_id)
  .fetch_optional(executor)
  .await
}

pub struct ClientSQLCommon {
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

impl PartialEq<ClientSQLRow> for ClientSQLCommon {
  fn eq(&self, other: &ClientSQLRow) -> bool {
    self.name == other.name
      && self.client_id == other.client_id
      && self.redirect_uris == other.redirect_uris
      && self.post_logout_redirect_uris == other.post_logout_redirect_uris
      && self.logo_uri == other.logo_uri
      && self.policy_uri == other.policy_uri
      && self.terms_of_service_uri == other.terms_of_service_uri
      && self.application_type == other.application_type
      && self.auth_method == other.auth_method
      && self.grant_types == other.grant_types
      && self.response_types == other.response_types
      && self.scopes == other.scopes
      && self.audience == other.audience
      && self.access_token_expires_in_seconds == other.access_token_expires_in_seconds
      && self.id_token_expires_in_seconds == other.id_token_expires_in_seconds
      && self.refresh_expires_in_seconds == other.refresh_expires_in_seconds
  }
}

// returns client and bool to indicate if its new or updated
pub async fn upsert_client(
  pool: &sqlx::AnyPool,
  client_upsert: ClientSQLCommon,
) -> sqlx::Result<(ClientSQLRow, bool)> {
  run_transaction(pool, |transaction| {
    Box::pin(async move {
      let client_option =
        get_client_by_client_id_internal(&mut **transaction, &client_upsert.client_id).await?;

      if let Some(client) = client_option {
        if client_upsert != client {
          let updated_client =
            update_client_internal(&mut **transaction, &client.client_id, client_upsert).await?;
          Ok((updated_client, false))
        } else {
          Ok((client, false))
        }
      } else {
        let new_client = create_client_internal(&mut **transaction, client_upsert).await?;
        Ok((new_client, true))
      }
    })
  })
  .await
}

async fn update_client_internal<'e, E>(
  executor: E,
  client_id: &str,
  client: ClientSQLCommon,
) -> sqlx::Result<ClientSQLRow>
where
  E: 'e + sqlx::Executor<'e, Database = sqlx::Any>,
{
  sqlx::query_as::<_, ClientSQLRow>(
    r#"UPDATE clients 
          SET 
            name = $1,
            redirect_uris = $2,
            post_logout_redirect_uris = $3,
            logo_uri = $4,
            client_uri = $5,
            policy_uri = $6,
            terms_of_service_uri = $7,
            application_type = $8,
            auth_method = $9,
            grant_types = $10,
            response_types = $11,
            scopes = $12,
            audience = $13,
            access_token_expires_in_seconds = $14,
            id_token_expires_in_seconds = $15,
            refresh_expires_in_seconds = $16,
            updated_at = $17
          WHERE client_id = $18
          RETURNING *;"#,
  )
  .bind(&client.name)
  .bind(&client.redirect_uris)
  .bind(&client.post_logout_redirect_uris)
  .bind(&client.logo_uri)
  .bind(&client.client_uri)
  .bind(&client.policy_uri)
  .bind(&client.terms_of_service_uri)
  .bind(&client.application_type)
  .bind(&client.auth_method)
  .bind(&client.grant_types)
  .bind(&client.response_types)
  .bind(&client.scopes)
  .bind(&client.audience)
  .bind(client.access_token_expires_in_seconds)
  .bind(client.id_token_expires_in_seconds)
  .bind(client.refresh_expires_in_seconds)
  .bind(chrono::Utc::now().timestamp())
  .bind(client_id)
  .fetch_one(executor)
  .await
}

async fn create_client_internal<'e, E>(
  executor: E,
  client: ClientSQLCommon,
) -> sqlx::Result<ClientSQLRow>
where
  E: 'e + sqlx::Executor<'e, Database = sqlx::Any>,
{
  sqlx::query_as::<_, ClientSQLRow>(
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
          RETURNING *;"#,
  )
  .bind(&client.name)
  .bind(&client.client_id)
  .bind(hex::encode(random_bytes(64)))
  .bind(&client.redirect_uris)
  .bind(&client.post_logout_redirect_uris)
  .bind(&client.logo_uri)
  .bind(&client.client_uri)
  .bind(&client.policy_uri)
  .bind(&client.terms_of_service_uri)
  .bind(&client.application_type)
  .bind(&client.auth_method)
  .bind(&client.grant_types)
  .bind(&client.response_types)
  .bind(&client.scopes)
  .bind(&client.audience)
  .bind(client.access_token_expires_in_seconds)
  .bind(client.id_token_expires_in_seconds)
  .bind(client.refresh_expires_in_seconds)
  .fetch_one(executor)
  .await
}
