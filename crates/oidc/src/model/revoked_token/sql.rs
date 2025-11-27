#[derive(sqlx::FromRow)]
pub struct RevokedTokenSQLRow {
  pub token: String,
  pub expires_at: i64,
  pub created_at: i64,
}

pub async fn is_token_revoked(pool: &sqlx::AnyPool, token: &str) -> sqlx::Result<bool> {
  let result = sqlx::query_scalar::<_, i64>(
    r#"SELECT COUNT(*)
    FROM revoked_tokens
    WHERE token = $1
    LIMIT 1;"#,
  )
  .bind(token)
  .fetch_one(pool)
  .await?;

  Ok(result > 0)
}

pub async fn revoke_token(pool: &sqlx::AnyPool, token: &str, expires_at: i64) -> sqlx::Result<()> {
  sqlx::query(
    r#"INSERT INTO revoked_tokens (token, expires_at)
    VALUES ($1, $2)
    ON CONFLICT (token) DO NOTHING;"#,
  )
  .bind(token)
  .bind(expires_at)
  .execute(pool)
  .await?;

  Ok(())
}

pub async fn cleanup_expired_tokens(pool: &sqlx::AnyPool) -> sqlx::Result<u64> {
  let now = chrono::Utc::now().timestamp();
  let result = sqlx::query(
    r#"DELETE FROM revoked_tokens
    WHERE expires_at < $1;"#,
  )
  .bind(now)
  .execute(pool)
  .await?;

  Ok(result.rows_affected())
}
