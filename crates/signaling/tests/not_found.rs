mod common;

use std::error::Error;

use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test(flavor = "multi_thread")]
async fn not_found() -> Result<(), Box<dyn Error>> {
  let (router, _config) = common::util::setup().await?;

  let response = router
    .oneshot(Request::builder().uri("/").body(Body::empty())?)
    .await
    .expect("failed to send request");

  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  Ok(())
}
