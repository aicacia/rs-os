#![forbid(unsafe_code)]
pub mod config;
pub mod connection;
pub mod entities;
pub mod migration;

pub use config::DatabaseConfig;
pub use connection::{close_database_connection, create_database_connection};
pub use entities::prelude::*;
