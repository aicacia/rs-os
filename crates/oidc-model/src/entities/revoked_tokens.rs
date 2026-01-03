use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "revoked_tokens")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false, unique)]
  pub token: String,
  pub expires_at: i64,
  pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Database operations for revoked_tokens
use sea_orm::Set;

pub async fn is_token_revoked(db: &DatabaseConnection, token: &str) -> Result<bool, DbErr> {
  let count = Entity::find()
    .filter(Column::Token.eq(token))
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
  let result = ActiveModel {
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
  let result = Entity::delete_many()
    .filter(Column::ExpiresAt.lt(now))
    .exec(db)
    .await?;

  Ok(result.rows_affected)
}
