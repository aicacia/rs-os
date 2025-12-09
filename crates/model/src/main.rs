use std::{
  fs::{File, create_dir_all},
  path::Path,
};

async fn connect_to_db(
  connect_opt: sea_orm::ConnectOptions,
) -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
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

  sea_orm::Database::connect(db_url).await
}

#[tokio::main]
async fn main() {
  dotenvy::dotenv().expect("Failed to load .env file");

  let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  let schema = std::env::var("DATABASE_SCHEMA").unwrap_or_else(|_| "public".to_owned());

  let connect_options = sea_orm::ConnectOptions::new(url)
    .set_schema_search_path(schema)
    .to_owned();

  let db = connect_to_db(connect_options)
    .await
    .expect("Failed to connect to database");

  db.get_schema_registry("os_model::entities::*")
    .sync(&db)
    .await
    .expect("failed to sync schema registry");
}
