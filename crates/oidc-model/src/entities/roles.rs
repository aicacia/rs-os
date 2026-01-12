use std::hash::{Hash, Hasher};

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "roles")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: i64,
  pub application_id: i64,
  pub uri: String,
  pub description: String,
  pub updated_at: i64,
  pub created_at: i64,
}

impl Hash for Model {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.id.hash(state);
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::roles_permissions::Entity")]
  RolesPermissions,
  #[sea_orm(has_many = "super::user_roles::Entity")]
  UserRoles,
  #[sea_orm(
    belongs_to = "super::applications::Entity",
    from = "Column::ApplicationId",
    to = "super::applications::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Applications,
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

impl Related<super::applications::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Applications.def()
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

// Database operations for roles
use super::{permissions, roles_permissions};

pub async fn get_role_permissions_by_role_id(
  db: &DatabaseConnection,
  role_id: i64,
) -> Result<Vec<permissions::Model>, DbErr> {
  // Query permissions through the roles_permissions join table
  let role_permissions = roles_permissions::Entity::find()
    .filter(roles_permissions::Column::RoleId.eq(role_id))
    .all(db)
    .await?;

  let permission_ids: Vec<i64> = role_permissions.iter().map(|rp| rp.permission_id).collect();

  if permission_ids.is_empty() {
    return Ok(Vec::new());
  }

  permissions::Entity::find()
    .filter(permissions::Column::Id.is_in(permission_ids))
    .all(db)
    .await
}

pub async fn list_roles_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<Model>, DbErr> {
  Entity::find()
    .inner_join(super::user_roles::Entity)
    .filter(super::user_roles::Column::UserId.eq(user_id))
    .all(db)
    .await
}
