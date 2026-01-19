use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_passwords")]
pub struct Model {
  #[sea_orm(primary_key, unique)]
  pub id: i64,
  pub user_id: i64,
  #[sea_orm(default_value = "1")]
  pub active: i64,
  pub encrypted_password: String,
  #[sea_orm(default_value = "0")]
  pub reset_required: i64,
  pub updated_at: i64,
  pub created_at: i64,
}

impl Model {
  pub fn is_reset_required(&self) -> bool {
    self.reset_required != 0
  }

  pub fn is_password_expired(&self, max_age_seconds: i64) -> bool {
    let now = chrono::Utc::now().timestamp();
    now - self.updated_at > max_age_seconds
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::users::Entity",
    from = "Column::UserId",
    to = "super::users::Column::Id",
    on_update = "NoAction",
    on_delete = "Cascade"
  )]
  Users,
}

impl Related<super::users::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Users.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

// Database operations for user_passwords
use sea_orm::Set;

pub async fn set_password_reset_required(
  db: &DatabaseConnection,
  password_id: i64,
  required: bool,
) -> Result<Model, DbErr> {
  let password = Entity::find_by_id(password_id).one(db).await?;

  if let Some(password) = password {
    let mut active_model: ActiveModel = password.into();
    active_model.reset_required = Set(if required { 1 } else { 0 });
    active_model.updated_at = Set(chrono::Utc::now().timestamp());
    active_model.update(db).await
  } else {
    Err(DbErr::RecordNotFound("Password not found".to_string()))
  }
}
