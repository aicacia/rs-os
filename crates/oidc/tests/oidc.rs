mod common;

use std::error::Error;

use axum::body::Body;
use http::{Request, StatusCode};
use scopeguard::defer;
use tower::ServiceExt;

use common::helper::{
  approve_client_for_user, create_admin_user, create_jwt_for_user, create_test_application,
  create_test_client, create_test_user,
};
use os_oidc_model::entities::clients;
use sea_orm::{ActiveModelTrait, Set};

#[tokio::test(flavor = "multi_thread")]
async fn test_jwks_endpoint() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .uri("/.well-known/jwks.json")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let jwks: serde_json::Value = serde_json::from_slice(&body)?;

  assert!(jwks.get("keys").is_some());

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_openid_configuration_endpoint() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .uri("/.well-known/openid-configuration")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let config: serde_json::Value = serde_json::from_slice(&body)?;

  assert!(config.get("issuer").is_some());
  assert!(config.get("authorization_endpoint").is_some());
  assert!(config.get("token_endpoint").is_some());
  assert!(config.get("userinfo_endpoint").is_some());

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_application_metadata_endpoint() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .uri("/.well-known/application-metadata.json")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let meta: serde_json::Value = serde_json::from_slice(&body)?;

  assert!(meta.get("urn").is_some());
  assert!(meta.get("client_id").is_some());
  assert!(meta.get("permissions").is_some());

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_authorize_get_endpoint() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri(&format!(
          "/authorize?client_id={}&response_type=code&response_mode=query&scope=openid&redirect_uri=http://localhost&state=state123",
          client.client_id
        ))
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::FOUND);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_introspect_endpoint_unauthorized() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/introspect")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::UNAUTHORIZED);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_device_authorize_endpoint_not_implemented() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/device-authorize")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::NOT_IMPLEMENTED);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_end_session_endpoint_missing_client_id() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(Request::builder().uri("/end-session").body(Body::empty())?)
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::BAD_REQUEST);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_token_endpoint_missing_body() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/token")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::BAD_REQUEST);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_revoke_endpoint_missing_auth() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/revoke")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("token=test&client_id=test"))?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_register_client_endpoint_unauthorized() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/register-client")
        .header("content-type", "application/json")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::UNAUTHORIZED);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_info_endpoint_unauthorized() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(Request::builder().uri("/user-info").body(Body::empty())?)
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::UNAUTHORIZED);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_endpoint_unauthorized() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .uri("/client?client_id=test")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::UNAUTHORIZED);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_allowed_endpoint_unauthorized() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .uri("/client-allowed?client_id=test")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::UNAUTHORIZED);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_approve_client_endpoint_unauthorized() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/approve-client?client_id=test")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::UNAUTHORIZED);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_authorize_client_endpoint_unauthorized() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/authorize-client")
        .header("content-type", "application/json")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::UNAUTHORIZED);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_introspect_endpoint_with_auth() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  approve_client_for_user(
    &database_connection,
    admin.id,
    &client.client_id,
    vec!["openid".to_string(), "profile".to_string()],
  )
  .await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/introspect")
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_register_client_endpoint_with_auth() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  approve_client_for_user(
    &database_connection,
    admin.id,
    &client.client_id,
    vec!["openid".to_string(), "profile".to_string()],
  )
  .await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let client_body = serde_json::json!({
    "name": "test_client_2",
    "client_id": "test_client_2",
    "redirect_uris": ["http://localhost:3000/callback"],
    "application_type": "web",
    "auth_method": "client_secret_basic",
    "grant_types": ["authorization_code", "refresh_token"],
    "response_types": ["code"],
    "scopes": ["openid", "profile"],
    "audience": [],
    "access_token_expires_in_seconds": 3600,
    "id_token_expires_in_seconds": 3600,
    "refresh_expires_in_seconds": 86400
  });

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/register-client")
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&client_body)?))?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::CREATED);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_info_endpoint_with_auth() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri("/user-info")
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let user_info: serde_json::Value = serde_json::from_slice(&body)?;

  assert!(user_info.get("sub").is_some());

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_endpoint_with_auth() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri(&format!("/client?client_id={}", client.client_id))
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_allowed_endpoint_with_auth() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  approve_client_for_user(
    &database_connection,
    admin.id,
    &client.client_id,
    vec!["openid".to_string(), "profile".to_string()],
  )
  .await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri(&format!("/client-allowed?client_id={}", client.client_id))
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_allowed_requires_reapprove_on_requested_scope_change()
-> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  approve_client_for_user(
    &database_connection,
    admin.id,
    &client.client_id,
    vec!["openid".to_string()],
  )
  .await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri(&format!(
          "/client-allowed?client_id={}&scope=openid%20profile",
          client.client_id
        ))
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::FORBIDDEN);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_allowed_requires_reapprove_on_client_scope_change()
-> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  approve_client_for_user(
    &database_connection,
    admin.id,
    &client.client_id,
    vec![
      "openid".to_string(),
      "profile".to_string(),
      "email".to_string(),
    ],
  )
  .await?;

  let mut updated_client: clients::ActiveModel = client.clone().into();
  updated_client.scopes = Set(r#"[\"openid\",\"profile\"]"#.to_string());
  updated_client.update(&database_connection).await?;

  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri(&format!("/client-allowed?client_id={}", client.client_id))
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::FORBIDDEN);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_approve_client_endpoint_with_auth() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri(&format!("/approve-client?client_id={}", client.client_id))
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_authorize_client_endpoint_with_auth() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  approve_client_for_user(
    &database_connection,
    admin.id,
    &client.client_id,
    vec!["openid".to_string(), "profile".to_string()],
  )
  .await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let auth_body = serde_json::json!({
    "client_id": client.client_id,
    "response_type": "code",
    "redirect_uri": "http://localhost:3000/callback",
    "scope": "openid profile",
    "code_challenge": "E9Mrozoa2owUedPyAPhnco2-_-ZdHua2LmYo9c-oNc",
    "code_challenge_method": "S256"
  });

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/authorize-client")
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&auth_body)?))?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(parts.status, StatusCode::OK);

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_info_response_contains_required_claims() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri("/user-info")
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();
  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let user_info: serde_json::Value = serde_json::from_slice(&body)?;

  assert!(user_info.get("sub").is_some(), "Missing 'sub' claim");
  assert!(
    user_info.get("username").is_some(),
    "Missing 'username' claim"
  );
  assert_eq!(
    user_info["sub"].as_str().unwrap(),
    format!("urn:os:sub:user:{}", admin.id),
    "Subject should match user URN"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_endpoint_returns_valid_client_object() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri(&format!("/client?client_id={}", client.client_id))
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();
  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let response_client: serde_json::Value = serde_json::from_slice(&body)?;

  assert_eq!(
    response_client["client_id"].as_str().unwrap(),
    client.client_id,
    "Client ID mismatch"
  );
  assert!(
    response_client.get("name").is_some(),
    "Client response should contain name"
  );
  assert!(
    response_client.get("redirect_uris").is_some(),
    "Client response should contain redirect_uris"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_openid_configuration_contains_all_required_endpoints() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .uri("/.well-known/openid-configuration")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();
  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let config: serde_json::Value = serde_json::from_slice(&body)?;

  assert!(config.get("issuer").is_some(), "Missing issuer endpoint");
  assert!(
    config.get("authorization_endpoint").is_some(),
    "Missing authorization_endpoint"
  );
  assert!(
    config.get("token_endpoint").is_some(),
    "Missing token_endpoint"
  );
  assert!(
    config.get("userinfo_endpoint").is_some(),
    "Missing userinfo_endpoint"
  );
  assert!(config.get("jwks_uri").is_some(), "Missing jwks_uri");
  assert!(
    config.get("response_types_supported").is_some(),
    "Missing response_types_supported"
  );
  assert!(
    config.get("grant_types_supported").is_some(),
    "Missing grant_types_supported"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_register_client_requires_admin_permissions() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let regular_user = create_test_user(&database_connection, None).await?;
  approve_client_for_user(
    &database_connection,
    regular_user.id,
    &client.client_id,
    vec!["openid".to_string()],
  )
  .await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &regular_user,
    &client.client_id,
    &[app.urn.as_str()],
    "openid",
  )
  .await?;

  let client_body = serde_json::json!({
    "name": "unauthorized_client",
    "client_id": "unauthorized_client",
    "redirect_uris": ["http://localhost:3000/callback"],
    "application_type": "web",
    "auth_method": "client_secret_basic",
    "grant_types": ["authorization_code"],
    "response_types": ["code"],
    "scopes": ["openid"],
    "audience": [],
    "access_token_expires_in_seconds": 3600,
    "id_token_expires_in_seconds": 3600,
    "refresh_expires_in_seconds": 86400
  });

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/register-client")
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&client_body)?))?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert!(
    parts.status != StatusCode::CREATED,
    "Unprivileged user should not be able to register client, got {:?}",
    parts.status
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_authorize_get_with_invalid_redirect_uri() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri(&format!(
          "/authorize?client_id={}&response_type=code&response_mode=query&scope=openid&redirect_uri=http://attacker.com&state=state123",
          client.client_id
        ))
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert!(
    parts.status == StatusCode::FOUND || parts.status == StatusCode::BAD_REQUEST,
    "Expected FOUND or BAD_REQUEST for invalid redirect URI, got {:?}",
    parts.status
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_authorize_get_with_unsupported_response_type() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri(&format!(
          "/authorize?client_id={}&response_type=token&response_mode=query&scope=openid&redirect_uri=http://localhost&state=state123",
          client.client_id
        ))
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert!(
    parts.status == StatusCode::FOUND || parts.status == StatusCode::BAD_REQUEST,
    "Expected FOUND or BAD_REQUEST for unsupported response_type, got {:?}",
    parts.status
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_authorize_client_validates_scopes() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let user = create_admin_user(&database_connection, app.id).await?;

  approve_client_for_user(
    &database_connection,
    user.id,
    &client.client_id,
    vec!["openid".to_string()],
  )
  .await?;

  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &user,
    &client.client_id,
    &[app.urn.as_str()],
    "openid",
  )
  .await?;

  let auth_body = serde_json::json!({
    "client_id": client.client_id,
    "response_type": "code",
    "redirect_uri": "http://localhost:3000/callback",
    "scope": "openid profile",
    "code_challenge": "E9Mrozoa2owUedPyAPhnco2-_-ZdHua2LmYo9c-oNc",
    "code_challenge_method": "S256"
  });

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/authorize-client")
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&auth_body)?))?,
    )
    .await
    .expect("failed to send request");

  let (_parts, body) = response.into_parts();

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  assert!(body.len() > 0, "Response should contain data");

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_info_rejects_malformed_jwt() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .uri("/user-info")
        .header(http::header::AUTHORIZATION, "Bearer malformed.jwt.token")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(
    parts.status,
    StatusCode::UNAUTHORIZED,
    "Should reject malformed JWT"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user_info_rejects_missing_bearer_token() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .uri("/user-info")
        .header(http::header::AUTHORIZATION, "NotABearerToken")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(
    parts.status,
    StatusCode::UNAUTHORIZED,
    "Should reject non-Bearer authentication"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_client_endpoint_rejects_invalid_token() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;

  let response = router
    .oneshot(
      Request::builder()
        .uri(&format!("/client?client_id={}", client.client_id))
        .header(
          "authorization",
          "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(
    parts.status,
    StatusCode::UNAUTHORIZED,
    "Should reject invalid JWT signature"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_token_endpoint_with_missing_grant_type() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(
          "code=test&client_id=test&redirect_uri=http://localhost",
        ))?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(
    parts.status,
    StatusCode::BAD_REQUEST,
    "Should reject token request without grant_type"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_token_endpoint_with_unsupported_grant_type() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, _database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(
          "grant_type=implicit&code=test&client_id=test&redirect_uri=http://localhost",
        ))?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(
    parts.status,
    StatusCode::BAD_REQUEST,
    "Should reject unsupported grant_type"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_token_endpoint_with_invalid_code() -> Result<(), Box<dyn Error>> {
  let (teardown, router, _config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(format!(
          "grant_type=authorization_code&code=invalid_code&client_id={}&redirect_uri=http://localhost:3000/callback",
          client.client_id
        )))?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert!(
    parts.status == StatusCode::BAD_REQUEST || parts.status == StatusCode::UNAUTHORIZED,
    "Should reject invalid authorization code, got {:?}",
    parts.status
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_introspect_returns_token_metadata() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  approve_client_for_user(
    &database_connection,
    admin.id,
    &client.client_id,
    vec!["openid".to_string(), "profile".to_string()],
  )
  .await?;
  let token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid profile",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/introspect")
        .header(
          http::header::AUTHORIZATION,
          format!("Bearer {}", token.access_token),
        )
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, body) = response.into_parts();
  assert_eq!(parts.status, StatusCode::OK);

  let body = axum::body::to_bytes(body, usize::MAX).await?;
  let introspect_response: serde_json::Value = serde_json::from_slice(&body)?;

  assert!(
    introspect_response.get("active").is_some() || introspect_response.get("scope").is_some(),
    "Introspect response should contain token metadata"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_introspect_endpoint_with_invalid_token() -> Result<(), Box<dyn Error>> {
  let (teardown, router, config, database_connection) = common::util::setup().await?;
  defer! { teardown() }

  let app = create_test_application(&database_connection).await?;
  let client = create_test_client(&database_connection, &app.urn, Some("test_client")).await?;
  let admin = create_admin_user(&database_connection, app.id).await?;
  approve_client_for_user(
    &database_connection,
    admin.id,
    &client.client_id,
    vec!["openid".to_string()],
  )
  .await?;
  let _valid_token = create_jwt_for_user(
    &database_connection,
    &config,
    &admin,
    &client.client_id,
    &[app.urn.as_str()],
    "openid",
  )
  .await?;

  let response = router
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/introspect")
        .header(http::header::AUTHORIZATION, "Bearer invalid.token.here")
        .body(Body::empty())?,
    )
    .await
    .expect("failed to send request");

  let (parts, _body) = response.into_parts();

  assert_eq!(
    parts.status,
    StatusCode::UNAUTHORIZED,
    "Should reject introspect with invalid bearer token"
  );

  Ok(())
}
