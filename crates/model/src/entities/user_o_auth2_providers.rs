
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_o_auth2_providers")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub user_id: i64,
  #[sea_orm(primary_key, auto_increment = false)]
  pub o_auth2_provider_id: i64,
  pub email: String,
  pub updated_at: i64,
  pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::o_auth2_providers::Entity",
    from = "Column::OAuth2ProviderId",
    to = "super::o_auth2_providers::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  OAuth2Providers,
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Users,
}

impl Related<super::o_auth2_providers::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OAuth2Providers.def()
  }
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Users.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
