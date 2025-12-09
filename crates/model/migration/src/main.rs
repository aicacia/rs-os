use std::{
  fs::{File, create_dir_all},
  path::Path,
};

use sea_orm_migration::{
  prelude::*,
  sea_orm::{ConnectOptions, Database, DatabaseConnection},
};

async fn connect_to_db(connect_opt: ConnectOptions) -> Result<DatabaseConnection, sea_orm::DbErr> {
  let db_url = connect_opt.get_url();

  if db_url.starts_with("sqlite:") {
    log::info!("Initializing sqlite database: {}", db_url);
    let path = Path::new(&db_url["sqlite:".len()..]);
    if let Some(parent) = path.parent() {
      if !parent.as_os_str().is_empty() && !parent.exists() {
        log::info!("Creating database directory: {:?}", parent);
        match create_dir_all(parent) {
          Ok(_) => (),
          Err(e) => {
            log::error!("Failed to create database directory: {}", e);
            return Err(sea_orm::DbErr::Custom(e.to_string()));
          }
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

  Database::connect(db_url).await
}

#[tokio::main]
async fn main() {
  cli::run_cli_with_connection(os_model_migration::Migrator, connect_to_db).await;
}
