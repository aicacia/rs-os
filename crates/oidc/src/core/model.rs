use std::io;

use chrono::Utc;
use jsonwebtoken::Algorithm;
use os_oidc_model::entities::applications;
use os_oidc_model::entities::clients::{self, ActiveModel};
use os_oidc_model::entities::jwks::{create_jwk, generate_jwk, list_jwks};
use os_oidc_model::entities::permissions;
use os_oidc_model::entities::roles;
use os_oidc_model::entities::roles_permissions;
use os_oidc_model::entities::user_infos;
use os_oidc_model::entities::user_passwords;
use os_oidc_model::entities::user_roles;
use os_oidc_model::entities::users;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::config::AppConfig;
use crate::core::encryption::random_bytes;
use crate::core::helper::string_vec_to_json;
use crate::router::common::entity::Permission;

async fn ensure_jwk_exists(db: &DatabaseConnection, default_alg: Algorithm) -> io::Result<()> {
  let has_any = !list_jwks(db).await.map_err(io::Error::other)?.is_empty();
  if !has_any {
    let jwk = generate_jwk(default_alg).map_err(io::Error::other)?;
    let _ = create_jwk(db, jwk).await.map_err(io::Error::other)?;
  }
  Ok(())
}

async fn ensure_application_exists(
  db: &DatabaseConnection,
) -> io::Result<os_oidc_model::entities::applications::Model> {
  let now = Utc::now().timestamp();
  let app = applications::Entity::find()
    .one(db)
    .await
    .map_err(io::Error::other)?;

  match app {
    Some(a) => Ok(a),
    None => {
      let new_app = applications::ActiveModel {
        name: Set("OIDC Application".to_owned()),
        description: Set(Some("OIDC Application".to_owned())),
        active: Set(1),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      new_app.insert(db).await.map_err(io::Error::other)
    }
  }
}

async fn ensure_oidc_web_client_exists(
  db: &DatabaseConnection,
  config: &AppConfig,
  app: &os_oidc_model::entities::applications::Model,
) -> io::Result<os_oidc_model::entities::clients::Model> {
  let ui_url = config.ui_url();
  let client_id = ui_url.clone();

  let mut scopes = Permission::all()
    .into_iter()
    .map(|p| p.to_string())
    .collect::<Vec<_>>();

  scopes.extend_from_slice(&[
    "openid".to_string(),
    "profile".to_string(),
    "email".to_string(),
    "address".to_string(),
    "phone".to_string(),
    "offline".to_string(),
  ]);

  let mut model: ActiveModel = ActiveModel {
    application_id: Set(Some(app.id)),
    name: Set("OIDC UI".to_string()),
    client_id: Set(client_id),
    auth_method: Set("none".to_string()),
    application_type: Set("web".to_string()),
    grant_types: Set(string_vec_to_json(&vec![
      "password".to_string(),
      "authorization_code".to_string(),
      "refresh_token".to_string(),
    ])),
    response_types: Set(string_vec_to_json(&vec![
      "code".to_string(),
      "none".to_string(),
    ])),
    scopes: Set(string_vec_to_json(&scopes)),
    audience: Set(string_vec_to_json(&vec![ui_url.clone()])),
    access_token_expires_in_seconds: Set(config.token.expires_in_seconds as i64),
    id_token_expires_in_seconds: Set(config.token.expires_in_seconds as i64),
    refresh_expires_in_seconds: Set(config.token.refresh_expires_in_seconds as i64),
    active: Set(1),
    ..Default::default()
  };

  model.redirect_uris = Set(Some(string_vec_to_json(&vec![ui_url.clone()])));
  model.post_logout_redirect_uris = Set(Some(string_vec_to_json(&vec![ui_url.clone()])));

  let (client, _) = clients::upsert_client(db, model, random_bytes)
    .await
    .map_err(io::Error::other)?;

  Ok(client)
}

async fn ensure_admin_user_exists(
  db: &DatabaseConnection,
  app: &os_oidc_model::entities::applications::Model,
) -> io::Result<()> {
  let now = Utc::now().timestamp();

  // Ensure all permissions exist
  for permission_enum in Permission::all() {
    let permission_uri = permission_enum.to_string();
    let permission = permissions::Entity::find()
      .filter(permissions::Column::Uri.eq(&permission_uri))
      .filter(permissions::Column::ApplicationId.eq(app.id))
      .one(db)
      .await
      .map_err(io::Error::other)?;
    if permission.is_none() {
      let p = permissions::ActiveModel {
        application_id: Set(app.id),
        uri: Set(permission_uri),
        description: Set(format!("{} permission", permission_enum)),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      p.insert(db).await.map_err(io::Error::other)?;
    }
  }

  let role = roles::Entity::find()
    .filter(roles::Column::Uri.eq("admin"))
    .filter(roles::Column::ApplicationId.eq(app.id))
    .one(db)
    .await
    .map_err(io::Error::other)?;
  let role = match role {
    Some(r) => r,
    None => {
      let r = roles::ActiveModel {
        application_id: Set(app.id),
        uri: Set("admin".to_owned()),
        description: Set("Administrator role".to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      r.insert(db).await.map_err(io::Error::other)?
    }
  };

  // Link admin:* permission to the admin role
  let permission = permissions::Entity::find()
    .filter(permissions::Column::Uri.eq("admin:*"))
    .filter(permissions::Column::ApplicationId.eq(app.id))
    .one(db)
    .await
    .map_err(io::Error::other)?;
  if let Some(permission) = permission {
    let rp_exists = roles_permissions::Entity::find()
      .filter(roles_permissions::Column::RoleId.eq(role.id))
      .filter(roles_permissions::Column::PermissionId.eq(permission.id))
      .one(db)
      .await
      .map_err(io::Error::other)?;
    if rp_exists.is_none() {
      let rp = roles_permissions::ActiveModel {
        role_id: Set(role.id),
        permission_id: Set(permission.id),
        created_at: Set(now),
        updated_at: Set(now),
      };
      rp.insert(db).await.map_err(io::Error::other)?;
    }
  }

  let user = users::Entity::find()
    .filter(users::Column::Username.eq("admin"))
    .one(db)
    .await
    .map_err(io::Error::other)?;
  let user = match user {
    Some(u) => u,
    None => {
      let u = users::ActiveModel {
        username: Set("admin".to_owned()),
        active: Set(1),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      u.insert(db).await.map_err(io::Error::other)?
    }
  };

  let ui = user_infos::Entity::find_by_id(user.id)
    .one(db)
    .await
    .map_err(io::Error::other)?;
  if ui.is_none() {
    let info = user_infos::ActiveModel {
      user_id: Set(user.id),
      nickname: Set(Some("admin".to_owned())),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    };
    info.insert(db).await.map_err(io::Error::other)?;
  }

  let ur_exists = user_roles::Entity::find()
    .filter(user_roles::Column::UserId.eq(user.id))
    .filter(user_roles::Column::RoleId.eq(role.id))
    .one(db)
    .await
    .map_err(io::Error::other)?;
  if ur_exists.is_none() {
    let ur = user_roles::ActiveModel {
      user_id: Set(user.id),
      role_id: Set(role.id),
      created_at: Set(now),
      updated_at: Set(now),
    };
    ur.insert(db).await.map_err(io::Error::other)?;
  }

  let pw_exists = user_passwords::Entity::find()
    .filter(user_passwords::Column::UserId.eq(user.id))
    .filter(user_passwords::Column::Active.ne(0))
    .one(db)
    .await
    .map_err(io::Error::other)?;
  if pw_exists.is_none() {
    let pw = user_passwords::ActiveModel {
      user_id: Set(user.id),
      // The password is "admin" hashed with Argon2id
      encrypted_password: Set(
        "$argon2id$v=19$m=19,t=2,p=1$cmc5ZXVXT1N0RmxjZFR1NQ$/0nLLEJDUFjP/lO6UhUHlzvL6Zlz1NO8BW+XdMNTG3c"
          .to_owned(),
      ),
      active: Set(1),
      created_at: Set(now),
      updated_at: Set(now),
      ..Default::default()
    };
    pw.insert(db).await.map_err(io::Error::other)?;
  }

  Ok(())
}

pub async fn init(db: &DatabaseConnection, config: &AppConfig) -> io::Result<()> {
  ensure_jwk_exists(db, config.token.default_jwt_algorithm).await?;
  let app = ensure_application_exists(db).await?;
  let _client = ensure_oidc_web_client_exists(db, config, &app).await?;
  ensure_admin_user_exists(db, &app).await?;
  Ok(())
}
