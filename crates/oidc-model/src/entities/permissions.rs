use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "permissions")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub client_id: i64,
  pub uri: String,
  pub description: String,
  pub updated_at: i64,
  pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::roles_permissions::Entity")]
  RolesPermissions,
  #[sea_orm(
    belongs_to = "super::clients::Entity",
    from = "Column::ClientId",
    to = "super::clients::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Clients,
}

impl Related<super::roles_permissions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::RolesPermissions.def()
  }
}

impl Related<super::clients::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Clients.def()
  }
}

impl Related<super::roles::Entity> for Entity {
  fn to() -> RelationDef {
    super::roles_permissions::Relation::Roles.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::roles_permissions::Relation::Permissions.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}
