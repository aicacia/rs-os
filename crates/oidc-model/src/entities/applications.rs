use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "applications")]
pub struct Model {
  #[sea_orm(primary_key, unique)]
  pub id: i64,
  #[sea_orm(default_value = "1")]
  pub active: i64,
  pub name: String,
  pub description: Option<String>,
  pub updated_at: i64,
  pub created_at: i64,
}

impl Model {
  pub fn is_active(&self) -> bool {
    self.active != 0
  }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::clients::Entity")]
  Clients,
  #[sea_orm(has_many = "super::roles::Entity")]
  Roles,
  #[sea_orm(has_many = "super::permissions::Entity")]
  Permissions,
}

impl Related<super::clients::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Clients.def()
  }
}

impl Related<super::roles::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Roles.def()
  }
}

impl Related<super::permissions::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Permissions.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}

pub async fn get_application_by_id(
  db: &DatabaseConnection,
  application_id: i64,
) -> Result<Option<Model>, DbErr> {
  Entity::find()
    .filter(Column::Id.eq(application_id))
    .one(db)
    .await
}

pub async fn list_applications(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
  Entity::find().all(db).await
}
