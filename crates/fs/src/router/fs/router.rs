use axum::{
  body::Bytes,
  extract::{Multipart, Path, Query, State},
  http::{HeaderMap, StatusCode, header},
  response::IntoResponse,
};

use os_api::{Authorization, BasicClaims, error::HttpError};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncReadExt;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::router::{
  entity::RouterState,
  fs::{
    constants::TAG,
    entity::{DeleteResponse, ListQuery, ListResponse, ObjectMetadata, UploadResponse},
  },
};

#[utoipa::path(
  get,
  path = "/files",
  tags = [TAG],
  params(ListQuery),
  responses(
    (status = 200, content_type = "application/json", body = ListResponse),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body =  HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["fs:read"])
  )
)]
pub async fn list(
  State(state): State<RouterState>,
  _user_authorization: Authorization<BasicClaims>,
  Query(ListQuery {
    prefix,
    max_keys,
    continuation_token: _continuation_token,
  }): Query<ListQuery>,
) -> Result<impl IntoResponse, HttpError> {
  let max_keys = max_keys.unwrap_or(1000).min(1000);
  let base_dir = PathBuf::from(&state.config.data_dir);
  let prefix_str = prefix.as_deref().unwrap_or("");
  let search_path = base_dir.join(prefix_str);

  let mut objects = Vec::new();

  if search_path.exists() {
    let mut entries = fs::read_dir(&search_path).await.map_err(|e| {
      HttpError::internal_error().with_application_error(format!("Failed to read directory: {}", e))
    })?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
      HttpError::internal_error().with_application_error(format!("Failed to read entry: {}", e))
    })? {
      let path = entry.path();
      if path.is_file() {
        let metadata = fs::metadata(&path).await.map_err(|e| {
          HttpError::internal_error()
            .with_application_error(format!("Failed to read metadata: {}", e))
        })?;

        let key = path
          .strip_prefix(&base_dir)
          .unwrap_or(&path)
          .to_string_lossy()
          .to_string();

        let last_modified = metadata
          .modified()
          .ok()
          .and_then(|t| {
            chrono::DateTime::<chrono::Utc>::from(t)
              .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
              .into()
          })
          .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        objects.push(ObjectMetadata {
          key,
          size: metadata.len(),
          last_modified,
          etag: format!(
            "\"{}_{}\"",
            metadata.len(),
            metadata
              .modified()
              .ok()
              .map(|t| t
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs())
              .unwrap_or(0)
          ),
          content_type: mime_guess::from_path(&path).first().map(|m| m.to_string()),
        });

        if objects.len() >= max_keys as usize {
          break;
        }
      }
    }
  }

  let response = ListResponse {
    objects,
    is_truncated: false,
    next_continuation_token: None,
  };

  Ok(axum::Json(response).into_response())
}

#[utoipa::path(
  get,
  path = "/files/{*key}",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/octet-stream", description = "Object data"),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["fs:read"])
  )
)]
pub async fn get_object(
  State(state): State<RouterState>,
  _user_authorization: Authorization<BasicClaims>,
  Path(key): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
  let base_dir = PathBuf::from(&state.config.data_dir);
  let file_path = base_dir.join(&key);

  // Prevent path traversal attacks
  if !file_path.starts_with(&base_dir) {
    return Err(HttpError::bad_request().with_application_error("Invalid file path"));
  }

  if !file_path.exists() {
    return Err(HttpError::not_found().with_application_error("File not found"));
  }

  let metadata = fs::metadata(&file_path).await.map_err(|e| {
    HttpError::internal_error().with_application_error(format!("Failed to read metadata: {}", e))
  })?;

  let mut file = fs::File::open(&file_path).await.map_err(|e| {
    HttpError::internal_error().with_application_error(format!("Failed to open file: {}", e))
  })?;

  let mut contents = Vec::new();
  file.read_to_end(&mut contents).await.map_err(|e| {
    HttpError::internal_error().with_application_error(format!("Failed to read file: {}", e))
  })?;

  let mut headers = HeaderMap::new();

  let content_type = mime_guess::from_path(&file_path)
    .first()
    .map(|m| m.to_string())
    .unwrap_or_else(|| "application/octet-stream".to_string());

  headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());

  if let Some(filename) = file_path.file_name() {
    headers.insert(
      header::CONTENT_DISPOSITION,
      format!("attachment; filename=\"{}\"", filename.to_string_lossy())
        .parse()
        .unwrap(),
    );
  }

  let etag = format!(
    "\"{}_{}\"",
    metadata.len(),
    metadata
      .modified()
      .ok()
      .map(|t| t
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
      .unwrap_or(0)
  );
  headers.insert(header::ETAG, etag.parse().unwrap());

  Ok((StatusCode::OK, headers, Bytes::from(contents)))
}

#[utoipa::path(
  head,
  path = "/files/{*key}",
  tags = [TAG],
  responses(
    (status = 200, description = "Object metadata in headers"),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["fs:read"])
  )
)]
pub async fn head_object(
  State(state): State<RouterState>,
  _user_authorization: Authorization<BasicClaims>,
  Path(key): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
  let base_dir = PathBuf::from(&state.config.data_dir);
  let file_path = base_dir.join(&key);

  // Prevent path traversal attacks
  if !file_path.starts_with(&base_dir) {
    return Err(HttpError::bad_request().with_application_error("Invalid file path"));
  }

  if !file_path.exists() {
    return Err(HttpError::not_found().with_application_error("File not found"));
  }

  let metadata = fs::metadata(&file_path).await.map_err(|e| {
    HttpError::internal_error().with_application_error(format!("Failed to read metadata: {}", e))
  })?;

  let mut headers = HeaderMap::new();

  let content_type = mime_guess::from_path(&file_path)
    .first()
    .map(|m| m.to_string())
    .unwrap_or_else(|| "application/octet-stream".to_string());

  headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
  headers.insert(
    header::CONTENT_LENGTH,
    metadata.len().to_string().parse().unwrap(),
  );

  let etag = format!(
    "\"{}_{}\"",
    metadata.len(),
    metadata
      .modified()
      .ok()
      .map(|t| t
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
      .unwrap_or(0)
  );
  headers.insert(header::ETAG, etag.parse().unwrap());

  if let Ok(modified) = metadata.modified() {
    if let Ok(datetime) = chrono::DateTime::<chrono::Utc>::from(modified)
      .format("%a, %d %b %Y %H:%M:%S GMT")
      .to_string()
      .parse()
    {
      headers.insert(header::LAST_MODIFIED, datetime);
    }
  }

  Ok((StatusCode::OK, headers))
}

#[utoipa::path(
  post,
  path = "/files/{*key}",
  tags = [TAG],
  request_body(content_type = "multipart/form-data", content = inline(UploadForm)),
  responses(
    (status = 201, content_type = "application/json", body = UploadResponse),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["fs:write"])
  )
)]
pub async fn upload_object(
  State(state): State<RouterState>,
  _user_authorization: Authorization<BasicClaims>,
  mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {
  let mut key: Option<String> = None;
  let mut file_data: Option<Bytes> = None;

  while let Some(field) = multipart.next_field().await.map_err(|e| {
    HttpError::bad_request()
      .with_application_error(format!("Failed to read multipart field: {}", e))
  })? {
    let field_name = field.name().unwrap_or("").to_string();

    match field_name.as_str() {
      "key" => {
        key = Some(field.text().await.map_err(|e| {
          HttpError::bad_request()
            .with_application_error(format!("Failed to read key field: {}", e))
        })?);
      }
      "file" => {
        file_data = Some(field.bytes().await.map_err(|e| {
          HttpError::bad_request()
            .with_application_error(format!("Failed to read file data: {}", e))
        })?);
      }
      _ => {}
    }
  }

  let key =
    key.ok_or_else(|| HttpError::bad_request().with_application_error("Missing 'key' field"))?;
  let file_data = file_data
    .ok_or_else(|| HttpError::bad_request().with_application_error("Missing 'file' field"))?;

  let base_dir = PathBuf::from(&state.config.data_dir);
  let file_path = base_dir.join(&key);

  // Prevent path traversal attacks
  if !file_path.starts_with(&base_dir) {
    return Err(HttpError::bad_request().with_application_error("Invalid file path"));
  }

  // Create parent directories if they don't exist
  if let Some(parent) = file_path.parent() {
    fs::create_dir_all(parent).await.map_err(|e| {
      HttpError::internal_error()
        .with_application_error(format!("Failed to create directories: {}", e))
    })?;
  }

  fs::write(&file_path, &file_data).await.map_err(|e| {
    HttpError::internal_error().with_application_error(format!("Failed to write file: {}", e))
  })?;

  let metadata = fs::metadata(&file_path).await.map_err(|e| {
    HttpError::internal_error().with_application_error(format!("Failed to read metadata: {}", e))
  })?;

  let etag = format!(
    "\"{}_{}\"",
    metadata.len(),
    metadata
      .modified()
      .ok()
      .map(|t| t
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
      .unwrap_or(0)
  );

  let response = UploadResponse {
    key: key.clone(),
    etag,
    size: file_data.len() as u64,
  };

  Ok((StatusCode::CREATED, axum::Json(response)))
}

#[utoipa::path(
  put,
  path = "/files/{*key}",
  tags = [TAG],
  request_body(content_type = "application/octet-stream", content = inline(Vec::<u8>)),
  responses(
    (status = 201, content_type = "application/json", body = UploadResponse),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["fs:write"])
  )
)]
pub async fn put_object(
  State(state): State<RouterState>,
  _user_authorization: Authorization<BasicClaims>,
  Path(key): Path<String>,
  body: Bytes,
) -> Result<impl IntoResponse, HttpError> {
  let base_dir = PathBuf::from(&state.config.data_dir);
  let file_path = base_dir.join(&key);

  // Prevent path traversal attacks
  if !file_path.starts_with(&base_dir) {
    return Err(HttpError::bad_request().with_application_error("Invalid file path"));
  }

  if let Some(parent) = file_path.parent() {
    fs::create_dir_all(parent).await.map_err(|e| {
      HttpError::internal_error()
        .with_application_error(format!("Failed to create directories: {}", e))
    })?;
  }

  fs::write(&file_path, &body).await.map_err(|e| {
    HttpError::internal_error().with_application_error(format!("Failed to write file: {}", e))
  })?;

  let metadata = fs::metadata(&file_path).await.map_err(|e| {
    HttpError::internal_error().with_application_error(format!("Failed to read metadata: {}", e))
  })?;

  let etag = format!(
    "\"{}_{}\"",
    metadata.len(),
    metadata
      .modified()
      .ok()
      .map(|t| t
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
      .unwrap_or(0)
  );

  let response = UploadResponse {
    key: key.clone(),
    etag,
    size: body.len() as u64,
  };

  Ok((StatusCode::CREATED, axum::Json(response)))
}

#[utoipa::path(
  delete,
  path = "/files/{*key}",
  tags = [TAG],
  responses(
    (status = 200, content_type = "application/json", body = DeleteResponse),
    (status = 400, content_type = "application/json", body = HttpError),
    (status = 401, content_type = "application/json", body = HttpError),
    (status = 403, content_type = "application/json", body = HttpError),
    (status = 404, content_type = "application/json", body = HttpError),
    (status = 500, content_type = "application/json", body = HttpError),
  ),
  security(
    ("Authorization" = ["fs:write"])
  )
)]
pub async fn delete_object(
  State(state): State<RouterState>,
  _user_authorization: Authorization<BasicClaims>,
  Path(key): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
  let base_dir = PathBuf::from(&state.config.data_dir);
  let file_path = base_dir.join(&key);

  // Prevent path traversal attacks
  if !file_path.starts_with(&base_dir) {
    return Err(HttpError::bad_request().with_application_error("Invalid file path"));
  }

  let deleted = if file_path.exists() {
    fs::remove_file(&file_path).await.map_err(|e| {
      HttpError::internal_error().with_application_error(format!("Failed to delete file: {}", e))
    })?;
    true
  } else {
    false
  };

  let response = DeleteResponse {
    key: key.clone(),
    deleted,
  };

  Ok(axum::Json(response).into_response())
}

/// Multipart form data for file upload
#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
struct UploadForm {
  file: Vec<u8>,
}

pub fn create_router(state: RouterState) -> OpenApiRouter {
  OpenApiRouter::new()
    .routes(routes!(list))
    .routes(routes!(get_object))
    .routes(routes!(head_object))
    .routes(routes!(upload_object))
    .routes(routes!(put_object))
    .routes(routes!(delete_object))
    .with_state(state)
}
