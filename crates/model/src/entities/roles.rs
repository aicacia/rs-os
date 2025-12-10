
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "roles")]
pub struct Model {
  #[sea_orm(primary_key, unique)]
  pub id: i64,
  #[sea_orm(unique)]
  pub uri: String,
  pub description: String,
  pub updated_at: i64,
  pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::roles_permissions::Entity")]
  RolesPermissions,
  #[sea_orm(has_many = "super::user_roles::Entity")]
  UserRoles,
}

impl Related<super::roles_permissions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::RolesPermissions.def()
  }
}

impl Related<super::user_roles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::UserRoles.def()
  }
}

impl Related<super::permissions::Entity> for Entity {
  fn to() -> RelationDef {
    super::roles_permissions::Relation::Permissions.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::roles_permissions::Relation::Roles.def().rev())
  }
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    super::user_roles::Relation::Users.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::user_roles::Relation::Roles.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}
