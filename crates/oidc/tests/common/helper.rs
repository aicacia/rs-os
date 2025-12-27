use std::error::Error;

use chrono::Utc;
use os_model::entities::{
  clients, jwks::get_jwk_for_sign_and_verify, permissions, roles, roles_permissions, user_roles,
  users,
};
use os_oidc::router::common::entity::{Permission, Token};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use os_model::entities::users::upsert_user_client;
use os_oidc::config::AppConfig;
use os_oidc::router::common::helper::create_user_token;

pub async fn create_test_user(
  db: &DatabaseConnection,
  username: Option<&str>,
) -> Result<users::Model, Box<dyn Error>> {
  let username = match username {
    Some(name) => name.to_string(),
    None => format!("test_{}", uuid::Uuid::new_v4()),
  };
  let user = users::create_user(db, &username).await?;
  Ok(user)
}

pub async fn create_test_role_with_permissions(
  db: &DatabaseConnection,
  role_uri: &str,
  permission_uris: Vec<&str>,
) -> Result<(roles::Model, Vec<permissions::Model>), Box<dyn Error>> {
  let now = Utc::now().timestamp();

  let role = roles::ActiveModel {
    uri: Set(role_uri.to_owned()),
    description: Set(role_uri.to_owned()),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  let role = role.insert(db).await?;

  let mut permissions = Vec::new();
  for perm_uri in permission_uris {
    let existing_permission = permissions::Entity::find()
      .filter(permissions::Column::Uri.eq(perm_uri))
      .one(db)
      .await?;

    let permission = if let Some(perm) = existing_permission {
      perm
    } else {
      let perm = permissions::ActiveModel {
        uri: Set(perm_uri.to_owned()),
        description: Set(perm_uri.to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      perm.insert(db).await?
    };

    let permission_id = permission.id;
    permissions.push(permission);

    let role_perm = roles_permissions::ActiveModel {
      role_id: Set(role.id),
      permission_id: Set(permission_id),
      created_at: Set(now),
      updated_at: Set(now),
    };
    role_perm.insert(db).await?;
  }

  Ok((role, permissions))
}

pub async fn assign_role_to_user(
  db: &DatabaseConnection,
  user_id: i64,
  role_id: i64,
) -> Result<(), Box<dyn Error>> {
  let now = Utc::now().timestamp();

  let user_role = user_roles::ActiveModel {
    user_id: Set(user_id),
    role_id: Set(role_id),
    created_at: Set(now),
    updated_at: Set(now),
  };

  user_role.insert(db).await?;
  Ok(())
}

pub async fn create_admin_user(db: &DatabaseConnection) -> Result<users::Model, Box<dyn Error>> {
  let user = create_test_user(db, None).await?;

  let (admin_role, _) =
    create_test_role_with_permissions(db, "test_admin", vec![Permission::AdminAll.as_str()])
      .await?;

  assign_role_to_user(db, user.id, admin_role.id).await?;

  Ok(user)
}

pub async fn create_jwt_for_user(
  db: &DatabaseConnection,
  app_config: &AppConfig,
  user: &users::Model,
  client_id: &str,
  audience: &str,
  scope: &str,
) -> Result<Token, Box<dyn Error>> {
  let jwk_model = match get_jwk_for_sign_and_verify(db).await {
    Ok(Some(jwk)) => jwk,
    Ok(None) => {
      return Err("No valid JWK found for signing and verifying JWTs".into());
    }
    Err(e) => {
      return Err(format!("Error getting JWK: {}", e).into());
    }
  };

  let token = create_user_token(
    db,
    app_config,
    jwk_model,
    user.clone(),
    client_id.to_owned(),
    audience.to_owned(),
    scope.to_owned(),
    "urn:ietf:params:oauth:token-type:access_token".to_owned(),
  )
  .await
  .map_err(|e| format!("Error creating user token: {}", e))?;

  Ok(token)
}

pub async fn create_test_client(
  db: &DatabaseConnection,
  client_id_override: Option<&str>,
) -> Result<clients::Model, Box<dyn Error>> {
  let client_id = match client_id_override {
    Some(id) => id.to_string(),
    None => format!("test_client_{}", uuid::Uuid::new_v4()),
  };
  let now = Utc::now().timestamp();

  let client = clients::ActiveModel {
    client_id: Set(client_id),
    name: Set("Test Client".to_string()),
    client_secret: Set("test_secret".to_string()),
    redirect_uris: Set(Some(r#"["http://localhost:3000/callback"]"#.to_string())),
    post_logout_redirect_uris: Set(Some(r#"["http://localhost:3000"]"#.to_string())),
    logo_uri: Set(None),
    client_uri: Set(None),
    policy_uri: Set(None),
    terms_of_service_uri: Set(None),
    application_type: Set("web".to_string()),
    auth_method: Set("client_secret_basic".to_string()),
    grant_types: Set(r#"["authorization_code", "refresh_token"]"#.to_string()),
    response_types: Set(r#"["code"]"#.to_string()),
    scopes: Set(r#"["openid", "profile", "email"]"#.to_string()),
    audience: Set("".to_string()),
    access_token_expires_in_seconds: Set(3600),
    id_token_expires_in_seconds: Set(3600),
    refresh_expires_in_seconds: Set(86400),
    active: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };

  let client = client.insert(db).await?;
  Ok(client)
}

pub async fn approve_client_for_user(
  db: &DatabaseConnection,
  user_id: i64,
  client_id: &str,
  scopes: Vec<String>,
) -> Result<(), Box<dyn Error>> {
  upsert_user_client(db, user_id, client_id, scopes).await?;
  Ok(())
}
