use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Permission {
  #[serde(rename = "admin:*")]
  AdminAll,

  #[serde(rename = "client:read")]
  ClientRead,
  #[serde(rename = "client:write")]
  ClientWrite,
  #[serde(rename = "client:delete")]
  ClientDelete,

  #[serde(rename = "user:read")]
  UserRead,
  #[serde(rename = "user:write")]
  UserWrite,
  #[serde(rename = "user:delete")]
  UserDelete,
}

impl Permission {
  pub fn as_str(&self) -> &'static str {
    match self {
      Permission::AdminAll => "admin:*",
      Permission::ClientRead => "client:read",
      Permission::ClientWrite => "client:write",
      Permission::ClientDelete => "client:delete",
      Permission::UserRead => "user:read",
      Permission::UserWrite => "user:write",
      Permission::UserDelete => "user:delete",
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
      "client:write" => Ok(Permission::ClientWrite),
      "client:delete" => Ok(Permission::ClientDelete),
      "user:read" => Ok(Permission::UserRead),
      "user:write" => Ok(Permission::UserWrite),
      "user:delete" => Ok(Permission::UserDelete),
      _ => Err(format!("unknown permission: {}", s)),
    }
  }
}
