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
