mod common;

use std::error::Error;

use axum::body::Body;
use http::{Request, StatusCode};
use os_api::util::entity::{Health, Version};
use tower::ServiceExt;

#[tokio::test(flavor = "multi_thread")]
async fn health() -> Result<(), Box<dyn Error>> {
  let (router, _config) = common::util::setup().await?;

  let response = router
    .oneshot(Request::builder().uri("/health").body(Body::empty())?)
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let health: Health = serde_json::from_slice(&body)?;

  assert!(health.ok);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn version() -> Result<(), Box<dyn Error>> {
  let (router, _config) = common::util::setup().await?;

  let response = router
    .oneshot(Request::builder().uri("/version").body(Body::empty())?)
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let version: Version = serde_json::from_slice(&body)?;

  assert_eq!(version.version, env!("CARGO_PKG_VERSION"));

  Ok(())
}
