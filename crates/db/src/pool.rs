use std::{
  fs::{File, create_dir_all},
  future::Future,
  path::Path,
  pin::Pin,
  time::Duration,
};

use sqlx::{Executor, migrate::Migrator};

use crate::database_config::DatabaseConfig;

pub async fn create(
  database_config: &DatabaseConfig,
  sqlite_migrator: &Migrator,
  postgresql_migrator: &Migrator,
) -> Result<sqlx::AnyPool, sqlx::Error> {
  sqlx::any::install_default_drivers();

  if database_config.url.starts_with("sqlite:") {
    log::info!("Initializing sqlite database: {}", database_config.url);
    let path = Path::new(&database_config.url["sqlite:".len()..]);
    if let Some(parent) = path.parent() {
      if !parent.as_os_str().is_empty() && !parent.exists() {
        log::info!("Creating database directory: {:?}", parent);
        match create_dir_all(parent) {
          Ok(_) => (),
          Err(e) => {
            log::error!("Failed to create database directory: {}", e);
            return Err(sqlx::Error::Io(e));
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
          return Err(sqlx::Error::Io(e));
        }
      }
    }
  }

  let pool = sqlx::any::AnyPoolOptions::new()
    .min_connections(database_config.min_connections)
    .max_connections(database_config.max_connections)
    .acquire_timeout(Duration::from_secs(database_config.acquire_timeout))
    .idle_timeout(Duration::from_secs(database_config.idle_timeout))
    .max_lifetime(Duration::from_secs(database_config.max_lifetime))
    .after_connect(|conn, _meta| {
      Box::pin(async move {
        match conn.backend_name().to_lowercase().as_str() {
          "sqlite" => {
            conn
              .execute(
                "PRAGMA journal_mode = wal; PRAGMA synchronous = normal; PRAGMA foreign_keys = on;",
              )
              .await?;
          }
          _ => (),
        }
        Ok(())
      })
    })
    .connect(&database_config.url)
    .await?;

  if database_config.url.starts_with("sqlite:") {
    log::info!("Running migrations for sqlite");
    sqlite_migrator.run(&pool).await?;
  } else if database_config.url.starts_with("postgres:") {
    log::info!("Running migrations for postgres");
    postgresql_migrator.run(&pool).await?;
  }

  Ok(pool)
}

pub async fn run_transaction<T, F>(
  pool: &sqlx::AnyPool,
  transaction_fn: F,
) -> Result<T, sqlx::Error>
where
  F: for<'a> FnOnce(
    &'a mut sqlx::Transaction<'_, sqlx::Any>,
  ) -> Pin<Box<dyn Send + Future<Output = sqlx::Result<T>> + 'a>>,
{
  let mut transaction = pool.begin().await?;
  let result = match transaction_fn(&mut transaction).await {
    Ok(result) => result,
    Err(e) => match transaction.rollback().await {
      Ok(_) => return Err(e),
      Err(e2) => {
        log::error!("Failed to rollback transaction: {}", e2);
        return Err(e);
      }
    },
  };
  transaction.commit().await?;
  Ok(result)
}

pub async fn close(pool: sqlx::AnyPool) -> Result<(), sqlx::Error> {
  {
    let conn = pool.acquire().await?;
    match conn.backend_name().to_lowercase().as_str() {
      "sqlite" => {
        log::info!("Optimizing database");
        pool
          .execute("PRAGMA analysis_limit=400; PRAGMA optimize;")
          .await?;
        log::info!("Optimized database");
      }
      _ => {}
    }
  }
  pool.close().await;
  Ok(())
}
