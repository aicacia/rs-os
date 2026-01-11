use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user_emails")]
pub struct Model {
  #[sea_orm(primary_key, unique)]
  pub id: i64,
  pub user_id: i64,
  #[sea_orm(unique)]
  pub email: String,
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

// Database operations for user_emails
use sea_orm::{Set, TransactionTrait, sea_query::Expr};

pub async fn list_user_emails_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Vec<Model>, DbErr> {
  Entity::find()
    .filter(Column::UserId.eq(user_id))
    .all(db)
    .await
}

pub async fn get_user_primary_email_by_user_id(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<Model>, DbErr> {
  Entity::find()
    .filter(Column::UserId.eq(user_id))
    .filter(Column::Primary.eq(1))
    .one(db)
    .await
}

pub async fn get_user_email_by_id(
  db: &DatabaseConnection,
  email_id: i64,
) -> Result<Option<Model>, DbErr> {
  Entity::find_by_id(email_id).one(db).await
}

pub async fn create_user_email(
  db: &DatabaseConnection,
  user_id: i64,
  email: &str,
) -> Result<Model, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let email_model = ActiveModel {
    user_id: Set(user_id),
    email: Set(email.to_owned()),
    verified: Set(0),
    primary: Set(0),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  email_model.insert(db).await
}

pub async fn update_user_email_primary(
  db: &DatabaseConnection,
  user_id: i64,
  email_id: i64,
) -> Result<Option<Model>, DbErr> {
  let txn = db.begin().await?;

  // First, unset all other primary emails for this user
  let now = chrono::Utc::now().timestamp();
  Entity::update_many()
    .col_expr(Column::Primary, Expr::value(0))
    .col_expr(Column::UpdatedAt, Expr::value(now))
    .filter(Column::UserId.eq(user_id))
    .filter(Column::Primary.ne(0))
    .exec(&txn)
    .await?;

  // Then set the specified email as primary
  let email = Entity::find_by_id(email_id)
    .filter(Column::UserId.eq(user_id))
    .one(&txn)
    .await?;

  let result = if let Some(email) = email {
    let mut active_model: ActiveModel = email.into();
    active_model.primary = Set(1);
    active_model.updated_at = Set(now);
    Some(active_model.update(&txn).await?)
  } else {
    None
  };

  txn.commit().await?;
  Ok(result)
}

pub async fn verify_user_email(
  db: &DatabaseConnection,
  user_id: i64,
  email_id: i64,
) -> Result<Option<Model>, DbErr> {
  let email = Entity::find_by_id(email_id)
    .filter(Column::UserId.eq(user_id))
    .one(db)
    .await?;

  if let Some(email) = email {
    let mut active_model: ActiveModel = email.into();
    active_model.verified = Set(1);
    active_model.updated_at = Set(chrono::Utc::now().timestamp());
    Ok(Some(active_model.update(db).await?))
  } else {
    Ok(None)
  }
}

pub async fn delete_user_email(
  db: &DatabaseConnection,
  user_id: i64,
  email_id: i64,
) -> Result<Option<Model>, DbErr> {
  let email = Entity::find_by_id(email_id)
    .filter(Column::UserId.eq(user_id))
    .one(db)
    .await?;

  if let Some(email) = email {
    let email_clone = email.clone();
    Entity::delete_by_id(email_id).exec(db).await?;
    Ok(Some(email_clone))
  } else {
    Ok(None)
  }
}

pub async fn get_user_primary_email(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<Model>, DbErr> {
  Entity::find()
    .filter(Column::UserId.eq(user_id))
    .filter(Column::Primary.eq(1))
    .one(db)
    .await
}
