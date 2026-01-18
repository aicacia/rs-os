use std::{fmt, str::FromStr, sync::Arc};

use crate::config::AppConfig;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone)]
pub struct RouterState {
  pub config: Arc<AppConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Permission {
  #[serde(rename = "*")]
  All,

  #[serde(rename = "fs:*")]
  FsAll,

  #[serde(rename = "fs:read")]
  FsRead,
  #[serde(rename = "fs:write")]
  FsWrite,
}

impl Permission {
  pub fn as_str(&self) -> &'static str {
    match self {
      Permission::All => "*",
      Permission::FsAll => "fs:*",
      Permission::FsRead => "fs:read",
      Permission::FsWrite => "fs:write",
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
      "fs:*" => Ok(Permission::FsAll),
      "fs:read" => Ok(Permission::FsRead),
      "fs:write" => Ok(Permission::FsWrite),
      _ => Err(format!("unknown permission: {}", s)),
    }
  }
}
