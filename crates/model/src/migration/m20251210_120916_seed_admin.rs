use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs() as i64;
    manager
      .get_connection()
      .execute_unprepared(&format!(
        r#"
INSERT INTO "roles" ("uri", "description", "updated_at", "created_at") VALUES ('admin', 'Administrator role', {now}, {now});
INSERT INTO "permissions" ("uri", "description", "updated_at", "created_at") VALUES ('admin:*', 'Administer all resources', {now}, {now});
INSERT INTO "roles_permissions" ("role_id", "permission_id", "updated_at", "created_at") VALUES (1, 1, {now}, {now});

INSERT INTO "users" ("username", "active", "updated_at", "created_at") VALUES ('admin', 1, {now}, {now});
INSERT INTO "user_infos" ("user_id", "nickname", "updated_at", "created_at") VALUES (1, 'admin', {now}, {now});
INSERT INTO "user_roles" ("role_id", "user_id", "updated_at", "created_at") VALUES (1, 1, {now}, {now});
INSERT INTO "user_passwords" ("user_id", "encrypted_password", "active", "updated_at", "created_at") VALUES (1, '$argon2id$v=19$m=19,t=2,p=1$cmc5ZXVXT1N0RmxjZFR1NQ$/0nLLEJDUFjP/lO6UhUHlzvL6Zlz1NO8BW+XdMNTG3c', 1, {now}, {now});
        "#,
      ))
      .await?;
    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared(
        r#"
DELETE FROM "user_roles" WHERE "user_id" = 1;
DELETE FROM "user_passwords" WHERE "user_id" = 1;
DELETE FROM "user_infos" WHERE "user_id" = 1;
DELETE FROM "users" WHERE "username" = 'admin';
DELETE FROM "roles_permissions" WHERE "role_id" = 1;
DELETE FROM "permissions" WHERE "uri" = 'admin:*';
DELETE FROM "roles" WHERE "uri" = 'admin';
        "#,
      )
      .await?;
    Ok(())
  }
}
