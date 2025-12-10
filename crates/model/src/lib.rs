pub mod entities;
pub mod migration;

pub use entities::prelude::*;
pub use sea_orm_migration::MigratorTrait;

pub async fn create_database_connection(
  database_config: &os_db::database_config::DatabaseConfig,
) -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
  let database_connection = os_db::connection::create(database_config).await?;

  database_connection
    .get_schema_registry("os_model::entities::*")
    .sync(&database_connection)
    .await
    .expect("failed to sync schema registry");

  migration::Migrator::up(&database_connection, None).await?;

  Ok(database_connection)
}
