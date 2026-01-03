use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_clients")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub user_id: i64,
  #[sea_orm(primary_key, auto_increment = false)]
  pub client_id: String,
  pub allowed_scopes: String,
  pub updated_at: i64,
  pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::clients::Entity",
    from = "Column::ClientId",
    to = "super::clients::Column::ClientId",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Clients,
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Users,
}

impl Related<super::clients::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Clients.def()
  }
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Users.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
