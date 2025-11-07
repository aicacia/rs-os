pub static SQLITE_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite");
pub static POSTGRESQL_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/postgresql");
