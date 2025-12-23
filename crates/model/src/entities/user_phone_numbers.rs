use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_phone_numbers")]
pub struct Model {
  #[sea_orm(primary_key, unique)]
  pub id: i64,
  pub user_id: i64,
  #[sea_orm(unique)]
  pub phone_number: String,
  pub verified: i64,
  pub primary: i64,
  pub updated_at: i64,
  pub created_at: i64,
}

impl Model {
  pub fn is_verified(&self) -> bool {
    self.verified != 0
  }

  pub fn is_primary(&self) -> bool {
    self.primary != 0
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

// Database operations for user_phone_numbers
use sea_orm::{Set, TransactionTrait, sea_query::Expr};

pub async fn list_user_phone_numbers_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<Model>, DbErr> {
  Entity::find()
    .filter(Column::UserId.eq(user_id))
    .all(db)
    .await
}

pub async fn get_user_primary_phone_number_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<Model>, DbErr> {
  Entity::find()
    .filter(Column::UserId.eq(user_id))
    .filter(Column::Primary.eq(1))
    .one(db)
    .await
}

pub async fn get_user_phone_number_by_id(
  db: &DatabaseConnection,
  phone_id: i64,
) -> Result<Option<Model>, DbErr> {
  Entity::find_by_id(phone_id).one(db).await
}

pub async fn create_user_phone_number(
  db: &DatabaseConnection,
  user_id: i64,
  phone_number: &str,
) -> Result<Model, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let phone_model = ActiveModel {
    user_id: Set(user_id),
    phone_number: Set(phone_number.to_owned()),
    verified: Set(0),
    primary: Set(0),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  phone_model.insert(db).await
}

pub async fn update_user_phone_number_primary(
  db: &DatabaseConnection,
  user_id: i64,
  phone_id: i64,
) -> Result<Option<Model>, DbErr> {
  let txn = db.begin().await?;

  // First, unset all other primary phone numbers for this user
  let now = chrono::Utc::now().timestamp();
  Entity::update_many()
    .col_expr(Column::Primary, Expr::value(0))
    .col_expr(Column::UpdatedAt, Expr::value(now))
    .filter(Column::UserId.eq(user_id))
    .filter(Column::Primary.ne(0))
    .exec(&txn)
    .await?;

  // Then set the specified phone number as primary
  let phone = Entity::find_by_id(phone_id)
    .filter(Column::UserId.eq(user_id))
    .one(&txn)
    .await?;

  let result = if let Some(phone) = phone {
    let mut active_model: ActiveModel = phone.into();
    active_model.primary = Set(1);
    active_model.updated_at = Set(now);
    Some(active_model.update(&txn).await?)
  } else {
    None
  };

  txn.commit().await?;
  Ok(result)
}

pub async fn verify_user_phone_number(
  db: &DatabaseConnection,
  user_id: i64,
  phone_id: i64,
) -> Result<Option<Model>, DbErr> {
  let phone = Entity::find_by_id(phone_id)
    .filter(Column::UserId.eq(user_id))
    .one(db)
    .await?;

  if let Some(phone) = phone {
    let mut active_model: ActiveModel = phone.into();
    active_model.verified = Set(1);
    active_model.updated_at = Set(chrono::Utc::now().timestamp());
    Ok(Some(active_model.update(db).await?))
  } else {
    Ok(None)
  }
}

pub async fn delete_user_phone_number(
  db: &DatabaseConnection,
  user_id: i64,
  phone_id: i64,
) -> Result<Option<Model>, DbErr> {
  let phone = Entity::find_by_id(phone_id)
    .filter(Column::UserId.eq(user_id))
    .one(db)
    .await?;

  if let Some(phone) = phone {
    let phone_clone = phone.clone();
    Entity::delete_by_id(phone_id).exec(db).await?;
    Ok(Some(phone_clone))
  } else {
    Ok(None)
  }
}

pub async fn get_user_primary_phone_number(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<Model>, DbErr> {
  Entity::find()
    .filter(Column::UserId.eq(user_id))
    .filter(Column::Primary.eq(1))
    .one(db)
    .await
}
