pub mod entities;

pub use entities::prelude::*;

pub async fn create_database_connection(
  database_config: &os_db::database_config::DatabaseConfig,
) -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
  let database_connection = os_db::connection::create(database_config).await?;

  database_connection
    .get_schema_registry("os_model::entities::*")
    .sync(&database_connection)
    .await
    .expect("failed to sync schema registry");

  Ok(database_connection)
}
