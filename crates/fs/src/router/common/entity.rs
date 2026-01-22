use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Permission {
  #[serde(rename = "*")]
  All,

  #[serde(rename = "fs:*")]
  FSAll,

  #[serde(rename = "fs:client:*")]
  FSClientAll,
  #[serde(rename = "fs:client:read")]
  FSClientRead,
  #[serde(rename = "fs:client:write")]
  FSClientWrite,

  #[serde(rename = "fs:user:*")]
  FSUserAll,
  #[serde(rename = "fs:user:read")]
  FSUserRead,
  #[serde(rename = "fs:user:write")]
  FSUserWrite,
}

impl Permission {
  pub fn as_str(&self) -> &'static str {
    match self {
      Permission::All => "*",
      Permission::FSAll => "fs:*",
      Permission::FSClientAll => "fs:client:*",
      Permission::FSClientRead => "fs:client:read",
      Permission::FSClientWrite => "fs:client:write",
      Permission::FSUserAll => "fs:user:*",
      Permission::FSUserRead => "fs:user:read",
      Permission::FSUserWrite => "fs:user:write",
    }
  }
}

impl fmt::Display for Permission {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_str())
  }
}

impl FromStr for Permission {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "*" => Ok(Permission::All),
      "fs:*" => Ok(Permission::FSAll),
      "fs:client:*" => Ok(Permission::FSClientAll),
      "fs:client:read" => Ok(Permission::FSClientRead),
      "fs:client:write" => Ok(Permission::FSClientWrite),
      "fs:user:*" => Ok(Permission::FSUserAll),
      "fs:user:read" => Ok(Permission::FSUserRead),
      "fs:user:write" => Ok(Permission::FSUserWrite),
      _ => Err(format!("unknown permission: {}", s)),
    }
  }
}
