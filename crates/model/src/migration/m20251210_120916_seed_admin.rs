use crate::entities::{
  permissions, roles, roles_permissions, user_infos, user_passwords, user_roles, users,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();
    let now = Utc::now().timestamp();

    let txn = db.begin().await?;

    let permission = permissions::Entity::find()
      .filter(permissions::Column::Uri.eq("admin:*"))
      .one(&txn)
      .await?;
    let permission = match permission {
      Some(p) => p,
      None => {
        let p = permissions::ActiveModel {
          uri: Set("admin:*".to_owned()),
          description: Set("Administer all resources".to_owned()),
          created_at: Set(now),
          updated_at: Set(now),
          ..Default::default()
        };
        p.insert(&txn).await?
      }
    };

    let role = roles::Entity::find()
      .filter(roles::Column::Uri.eq("admin"))
      .one(&txn)
      .await?;
    let role = match role {
      Some(r) => r,
      None => {
        let r = roles::ActiveModel {
          uri: Set("admin".to_owned()),
          description: Set("Administrator role".to_owned()),
          created_at: Set(now),
          updated_at: Set(now),
          ..Default::default()
        };
        r.insert(&txn).await?
      }
    };

    let rp_exists = roles_permissions::Entity::find()
      .filter(roles_permissions::Column::RoleId.eq(role.id))
      .filter(roles_permissions::Column::PermissionId.eq(permission.id))
      .one(&txn)
      .await?;
    if rp_exists.is_none() {
      let rp = roles_permissions::ActiveModel {
        role_id: Set(role.id),
        permission_id: Set(permission.id),
        created_at: Set(now),
        updated_at: Set(now),
      };
      rp.insert(&txn).await?;
    }

    let user = users::Entity::find()
      .filter(users::Column::Username.eq("admin"))
      .one(&txn)
      .await?;
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
        u.insert(&txn).await?
      }
    };

    let ui = user_infos::Entity::find_by_id(user.id).one(&txn).await?;
    if ui.is_none() {
      let info = user_infos::ActiveModel {
        user_id: Set(user.id),
        nickname: Set(Some("admin".to_owned())),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      info.insert(&txn).await?;
    }

    let ur_exists = user_roles::Entity::find()
      .filter(user_roles::Column::UserId.eq(user.id))
      .filter(user_roles::Column::RoleId.eq(role.id))
      .one(&txn)
      .await?;
    if ur_exists.is_none() {
      let ur = user_roles::ActiveModel {
        user_id: Set(user.id),
        role_id: Set(role.id),
        created_at: Set(now),
        updated_at: Set(now),
      };
      ur.insert(&txn).await?;
    }

    let pw_exists = user_passwords::Entity::find()
      .filter(user_passwords::Column::UserId.eq(user.id))
      .filter(user_passwords::Column::Active.ne(0))
      .one(&txn)
      .await?;
    if pw_exists.is_none() {
      let pw = user_passwords::ActiveModel {
        user_id: Set(user.id),
        encrypted_password: Set(
          "$argon2id$v=19$m=19,t=2,p=1$cmc5ZXVXT1N0RmxjZFR1NQ$/0nLLEJDUFjP/lO6UhUHlzvL6Zlz1NO8BW+XdMNTG3c"
            .to_owned(),
        ),
        active: Set(1),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
      };
      pw.insert(&txn).await?;
    }

    txn.commit().await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();
    let txn = db.begin().await?;

    let role = roles::Entity::find()
      .filter(roles::Column::Uri.eq("admin"))
      .one(&txn)
      .await?;
    let permission = permissions::Entity::find()
      .filter(permissions::Column::Uri.eq("admin:*"))
      .one(&txn)
      .await?;
    let user = users::Entity::find()
      .filter(users::Column::Username.eq("admin"))
      .one(&txn)
      .await?;

    if let Some(u) = user.clone() {
      user_roles::Entity::delete_many()
        .filter(user_roles::Column::UserId.eq(u.id))
        .exec(&txn)
        .await?;
      user_passwords::Entity::delete_many()
        .filter(user_passwords::Column::UserId.eq(u.id))
        .exec(&txn)
        .await?;
      user_infos::Entity::delete_many()
        .filter(user_infos::Column::UserId.eq(u.id))
        .exec(&txn)
        .await?;
      users::Entity::delete_many()
        .filter(users::Column::Id.eq(u.id))
        .exec(&txn)
        .await?;
    }

    if let Some(r) = role.clone() {
      roles_permissions::Entity::delete_many()
        .filter(roles_permissions::Column::RoleId.eq(r.id))
        .exec(&txn)
        .await?;
    }

    if permission.is_some() {
      permissions::Entity::delete_many()
        .filter(permissions::Column::Uri.eq("admin:*"))
        .exec(&txn)
        .await?;
    }
    if role.is_some() {
      roles::Entity::delete_many()
        .filter(roles::Column::Uri.eq("admin"))
        .exec(&txn)
        .await?;
    }

    txn.commit().await?;
    Ok(())
  }
}
