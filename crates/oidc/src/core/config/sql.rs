use crate::core::config::entity::Config;

#[derive(Debug, sqlx::FromRow)]
pub struct ConfigSQLRow {
  pub key: String,
  pub value: String,
  pub updated_at: i64,
  pub created_at: i64,
}

impl Into<Config> for ConfigSQLRow {
  fn into(self) -> Config {
    Config {
      key: self.key,
      value: self.value,
      updated_at: chrono::DateTime::from_timestamp(self.updated_at, 0).unwrap_or_default(),
      created_at: chrono::DateTime::from_timestamp(self.created_at, 0).unwrap_or_default(),
    }
  }
}

pub async fn list_configs(pool: &sqlx::AnyPool) -> sqlx::Result<Vec<ConfigSQLRow>> {
  sqlx::query_as(r#"SELECT * FROM configs;"#)
    .fetch_all(pool)
    .await
}

pub async fn get_config_by_key(
  pool: &sqlx::AnyPool,
  key: impl Into<String>,
) -> sqlx::Result<Option<ConfigSQLRow>> {
  sqlx::query_as(r#"SELECT * FROM configs WHERE "key" = $1;"#)
    .bind(key.into())
    .fetch_optional(pool)
    .await
}

pub async fn upsert_config(
  pool: &sqlx::AnyPool,
  key: impl Into<String>,
  value: impl Into<String>,
) -> sqlx::Result<ConfigSQLRow> {
  sqlx::query_as(
    r#"INSERT INTO configs ("key", "value")
          VALUES ($1, $2, $3)
          ON CONFLICT ("key")
          DO UPDATE SET "value" = $2, "updated_at" = $5
          RETURNING *;"#,
  )
  .bind(key.into())
  .bind(value.into())
  .bind(chrono::Utc::now().timestamp())
  .fetch_one(pool)
  .await
}

pub async fn delete_config(
  pool: &sqlx::AnyPool,
  key: impl Into<String>,
) -> sqlx::Result<Option<ConfigSQLRow>> {
  sqlx::query_as(r#"DELETE FROM configs WHERE "key" = $1 RETURNING *;"#)
    .bind(key.into())
    .fetch_optional(pool)
    .await
}
