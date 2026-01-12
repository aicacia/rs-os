use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

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
