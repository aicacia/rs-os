use os_model::entities::{prelude::*, *};
use sea_orm::*;

// Type alias for backward compatibility
pub type RevokedTokenModel = revoked_tokens::Model;

pub async fn is_token_revoked(db: &DatabaseConnection, token: &str) -> Result<bool, DbErr> {
  let count = RevokedTokens::find()
    .filter(revoked_tokens::Column::Token.eq(token))
    .count(db)
    .await?;

  Ok(count > 0)
}

pub async fn revoke_token(
  db: &DatabaseConnection,
  token: &str,
  expires_at: i64,
) -> Result<(), DbErr> {
  let now = chrono::Utc::now().timestamp();

  // Try to insert, but ignore if it already exists
  let result = revoked_tokens::ActiveModel {
    token: Set(token.to_owned()),
    expires_at: Set(expires_at),
    created_at: Set(now),
  }
  .insert(db)
  .await;

  // Ignore unique constraint violations (duplicate tokens)
  match result {
    Ok(_) => Ok(()),
    Err(DbErr::RecordNotInserted) | Err(DbErr::Exec(_)) => {
      // Treat constraint violations as success since the token is already revoked
      Ok(())
    }
    Err(e) => Err(e),
  }
}

pub async fn cleanup_expired_tokens(db: &DatabaseConnection) -> Result<u64, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let result = RevokedTokens::delete_many()
    .filter(revoked_tokens::Column::ExpiresAt.lt(now))
    .exec(db)
    .await?;

  Ok(result.rows_affected)
}
