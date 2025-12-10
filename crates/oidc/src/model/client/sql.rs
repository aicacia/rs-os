// Re-export ORM types and functions for backward compatibility
pub use super::orm::{
  ClientCommon, ClientModel as ClientSQLRow, ClientModelExt, deactivate_client,
  get_client_by_client_id, list_clients, upsert_client,
};

// Legacy type alias
pub type ClientSQLCommon = ClientCommon;
