
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_infos")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false, unique)]
  pub user_id: i64,
  pub given_name: Option<String>,
  pub family_name: Option<String>,
  pub middle_name: Option<String>,
  pub nickname: Option<String>,
  pub profile_picture: Option<String>,
  pub website: Option<String>,
  pub gender: Option<String>,
  pub birthdate: Option<i64>,
  pub zone_info: Option<String>,
  pub locale: Option<String>,
  pub address: Option<String>,
  pub updated_at: i64,
  pub created_at: i64,
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

// Database operations for user_infos
use sea_orm::Set;

#[derive(Default)]
pub struct UserInfoUpdate {
  pub given_name: Option<String>,
  pub family_name: Option<String>,
  pub middle_name: Option<String>,
  pub nickname: Option<String>,
  pub profile_picture: Option<String>,
  pub website: Option<String>,
  pub gender: Option<String>,
  pub birthdate: Option<i64>,
  pub zone_info: Option<String>,
  pub locale: Option<String>,
  pub address: Option<String>,
}

pub async fn get_user_info_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<Model>, DbErr> {
  Entity::find_by_id(user_id).one(db).await
}

pub async fn update_user_info(
  db: &DatabaseConnection,
  user_id: i64,
  user_info: UserInfoUpdate,
) -> Result<Model, DbErr> {
  let existing = Entity::find_by_id(user_id)
    .one(db)
    .await?
    .ok_or(DbErr::RecordNotFound("UserInfo not found".to_string()))?;

  let mut active_model: ActiveModel = existing.into();

  if let Some(val) = user_info.given_name {
    active_model.given_name = Set(Some(val));
  }
  if let Some(val) = user_info.family_name {
    active_model.family_name = Set(Some(val));
  }
  if let Some(val) = user_info.middle_name {
    active_model.middle_name = Set(Some(val));
  }
  if let Some(val) = user_info.nickname {
    active_model.nickname = Set(Some(val));
  }
  if let Some(val) = user_info.profile_picture {
    active_model.profile_picture = Set(Some(val));
  }
  if let Some(val) = user_info.website {
    active_model.website = Set(Some(val));
  }
  if let Some(val) = user_info.gender {
    active_model.gender = Set(Some(val));
  }
  if let Some(val) = user_info.birthdate {
    active_model.birthdate = Set(Some(val));
  }
  if let Some(val) = user_info.zone_info {
    active_model.zone_info = Set(Some(val));
  }
  if let Some(val) = user_info.locale {
    active_model.locale = Set(Some(val));
  }
  if let Some(val) = user_info.address {
    active_model.address = Set(Some(val));
  }

  active_model.updated_at = Set(chrono::Utc::now().timestamp());
  active_model.update(db).await
}
