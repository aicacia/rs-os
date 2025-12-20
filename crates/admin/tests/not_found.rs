mod common;

use std::error::Error;

use axum::body::Body;
use http::{Request, StatusCode};
use scopeguard::defer;
use tower::ServiceExt;

#[tokio::test(flavor = "multi_thread")]
async fn not_found() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _pool) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(Request::builder().uri("/").body(Body::empty())?)
    .await
    .expect("failed to send request");

  assert_eq!(response.status(), StatusCode::NOT_FOUND);

  Ok(())
}
