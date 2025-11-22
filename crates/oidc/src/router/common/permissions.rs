use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
  #[serde(rename = "admin:*")]
  AdminAll,

  #[serde(rename = "client:read")]
  ClientRead,

  #[serde(rename = "client:create")]
  ClientCreate,

  #[serde(rename = "client:update")]
  ClientUpdate,

  #[serde(rename = "client:delete")]
  ClientDelete,
}

impl Permission {
  pub fn as_str(&self) -> &'static str {
    match self {
      Permission::AdminAll => "admin:*",
      Permission::ClientRead => "client:read",
      Permission::ClientCreate => "client:create",
      Permission::ClientUpdate => "client:update",
      Permission::ClientDelete => "client:delete",
    }
  }
}

impl std::fmt::Display for Permission {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

impl FromStr for Permission {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "admin:*" => Ok(Permission::AdminAll),
      "client:read" => Ok(Permission::ClientRead),
      "client:create" => Ok(Permission::ClientCreate),
      "client:update" => Ok(Permission::ClientUpdate),
      "client:delete" => Ok(Permission::ClientDelete),
      _ => Err(format!("unknown permission: {}", s)),
    }
  }
}
