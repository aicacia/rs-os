use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum Permission {
  #[serde(rename = "admin:*")]
  AdminAll,

  #[serde(rename = "fs:read")]
  FsRead,
  #[serde(rename = "fs:write")]
  FsWrite,
}

impl Permission {
  pub fn as_str(&self) -> &'static str {
    match self {
      Permission::AdminAll => "admin:*",
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
      "admin:*" => Ok(Permission::AdminAll),
      "fs:read" => Ok(Permission::FsRead),
      "fs:write" => Ok(Permission::FsWrite),
      _ => Err(format!("unknown permission: {}", s)),
    }
  }
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct ListQuery {
  pub prefix: Option<String>,
  pub max_keys: Option<u32>,
  pub continuation_token: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ListResponse {
  pub objects: Vec<ObjectMetadata>,
  pub is_truncated: bool,
  pub next_continuation_token: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ObjectMetadata {
  pub key: String,
  pub size: u64,
  pub last_modified: String,
  pub etag: String,
  pub content_type: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct UploadResponse {
  pub key: String,
  pub etag: String,
  pub size: u64,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteResponse {
  pub key: String,
  pub deleted: bool,
}
