
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

// Database operations for user_o_auth2_providers
use sea_orm::Set;
use super::o_auth2_providers;

pub async fn get_user_oauth2_providers(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<(Model, Option<o_auth2_providers::Model>)>, DbErr> {
  Entity::find()
    .filter(Column::UserId.eq(user_id))
    .find_also_related(o_auth2_providers::Entity)
    .all(db)
    .await
}

pub async fn get_user_oauth2_provider_by_id(
  db: &DatabaseConnection,
  user_id: i64,
  provider_id: i64,
) -> Result<Option<(Model, Option<o_auth2_providers::Model>)>, DbErr> {
  Entity::find()
    .filter(Column::UserId.eq(user_id))
    .filter(Column::OAuth2ProviderId.eq(provider_id))
    .find_also_related(o_auth2_providers::Entity)
    .one(db)
    .await
}

pub async fn link_user_oauth2_provider(
  db: &DatabaseConnection,
  user_id: i64,
  oauth2_provider_id: i64,
  _name: &str,
  email: &str,
) -> Result<Model, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let model = ActiveModel {
    user_id: Set(user_id),
    o_auth2_provider_id: Set(oauth2_provider_id),
    email: Set(email.to_owned()),
    created_at: Set(now),
    updated_at: Set(now),
  };
  model.insert(db).await
}

pub async fn delete_user_oauth2_provider(
  db: &DatabaseConnection,
  user_id: i64,
  oauth2_provider_id: i64,
) -> Result<Option<Model>, DbErr> {
  let provider = Entity::find_by_id((user_id, oauth2_provider_id))
    .one(db)
    .await?;

  if let Some(provider) = provider {
    let provider_clone = provider.clone();
    Entity::delete_by_id((user_id, oauth2_provider_id))
      .exec(db)
      .await?;
    Ok(Some(provider_clone))
  } else {
    Ok(None)
  }
}
