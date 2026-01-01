use std::{
  fs::{File, create_dir_all},
  path::Path,
  time::Duration,
};

use sea_orm::ConnectionTrait;
use sea_orm_migration::MigratorTrait;

use crate::{config::DatabaseConfig, migration::Migrator};

pub async fn create_database_connection(
  database_config: &DatabaseConfig,
) -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
  if database_config.url.starts_with("sqlite:") {
    log::info!("Initializing sqlite database: {}", database_config.url);
    let path = Path::new(&database_config.url["sqlite:".len()..]);
    if let Some(parent) = path.parent()
      && !parent.as_os_str().is_empty()
      && !parent.exists()
    {
      log::info!("Creating database directory: {:?}", parent);
      match create_dir_all(parent) {
        Ok(_) => (),
        Err(e) => {
          log::error!("Failed to create database directory: {}", e);
          return Err(sea_orm::DbErr::Custom(e.to_string()));
        }
      }
    }
    if !path.exists() {
      log::info!("Creating database file: {:?}", path);
      match File::create(path) {
        Ok(_) => (),
        Err(e) => {
          log::error!("Failed to create database file: {}", e);
          return Err(sea_orm::DbErr::Custom(e.to_string()));
        }
      }
    }
  }

  let mut connect_options = sea_orm::ConnectOptions::new(database_config.url.clone());
  connect_options
    .min_connections(database_config.min_connections)
    .max_connections(database_config.max_connections)
    .acquire_timeout(Duration::from_secs(database_config.acquire_timeout))
    .idle_timeout(Duration::from_secs(database_config.idle_timeout))
    .max_lifetime(Duration::from_secs(database_config.max_lifetime))
    .after_connect(|conn| {
      Box::pin(async move {
        if conn
          .get_database_backend()
          .as_str()
          .eq_ignore_ascii_case("sqlx-sqlite")
        {
          conn
            .execute_unprepared(
              "PRAGMA journal_mode = wal; PRAGMA synchronous = normal; PRAGMA foreign_keys = on;",
            )
            .await?;
        }
        Ok(())
      })
    });

  let database_connection = sea_orm::Database::connect(connect_options).await?;

  database_connection
    .get_schema_registry("os_oidc_model::entities::*")
    .sync(&database_connection)
    .await?;

  Migrator::up(&database_connection, None).await?;

  Ok(database_connection)
}

pub async fn close_database_connection(
  database_connection: sea_orm::DatabaseConnection,
) -> Result<(), sea_orm::DbErr> {
  {
    if database_connection
      .get_database_backend()
      .as_str()
      .eq_ignore_ascii_case("sqlx-sqlite")
    {
      database_connection
        .execute_unprepared("PRAGMA analysis_limit=400; PRAGMA optimize;")
        .await?;
    }
  }
  database_connection.close().await?;
  Ok(())
}
