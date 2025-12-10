
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
