use std::str::FromStr;

pub use os_api::claims::{BasicClaims, Claims};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Permission {
  #[serde(rename = "*")]
  All,

  #[serde(rename = "client:*")]
  ClientAll,
  #[serde(rename = "client:read")]
  ClientRead,
  #[serde(rename = "client:write")]
  ClientWrite,
  #[serde(rename = "client:delete")]
  ClientDelete,

  #[serde(rename = "user:*")]
  UserAll,
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
      Permission::All => "*",
      Permission::ClientAll => "client:*",
      Permission::ClientRead => "client:read",
      Permission::ClientWrite => "client:write",
      Permission::ClientDelete => "client:delete",
      Permission::UserAll => "user:*",
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
      "*" => Ok(Permission::All),
      "client:*" => Ok(Permission::ClientAll),
      "client:read" => Ok(Permission::ClientRead),
      "client:write" => Ok(Permission::ClientWrite),
      "client:delete" => Ok(Permission::ClientDelete),
      "user:*" => Ok(Permission::UserAll),
      "user:read" => Ok(Permission::UserRead),
      "user:write" => Ok(Permission::UserWrite),
      "user:delete" => Ok(Permission::UserDelete),
      _ => Err(format!("unknown permission: {}", s)),
    }
  }
}
