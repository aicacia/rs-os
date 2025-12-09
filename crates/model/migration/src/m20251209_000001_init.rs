use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Create JWKS table
    manager
      .create_table(
        Table::create()
          .table(Jwks::Table)
          .if_not_exists()
          .col(pk_auto(Jwks::Kid))
          .col(integer(Jwks::Active).not_null().default(1))
          .col(string(Jwks::Kty).not_null())
          .col(string(Jwks::Alg).not_null())
          .col(string(Jwks::Use).null())
          .col(string(Jwks::KeyOps).null())
          // RSA fields
          .col(string(Jwks::N).null())
          .col(string(Jwks::E).null())
          .col(string(Jwks::D).null())
          .col(string(Jwks::P).null())
          .col(string(Jwks::Q).null())
          .col(string(Jwks::Dp).null())
          .col(string(Jwks::Dq).null())
          .col(string(Jwks::Qi).null())
          // EC fields
          .col(string(Jwks::Crv).null())
          .col(string(Jwks::X).null())
          .col(string(Jwks::Y).null())
          .col(string(Jwks::DEc).null())
          // Symmetric fields
          .col(string(Jwks::K).null())
          // X.509 fields
          .col(string(Jwks::X5u).null())
          .col(string(Jwks::X5c).null())
          .col(string(Jwks::X5t).null())
          .col(string(Jwks::X5tS256).null())
          .col(big_integer(Jwks::UpdatedAt).not_null())
          .col(big_integer(Jwks::CreatedAt).not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("jwks_kid_unique_idx")
          .table(Jwks::Table)
          .col(Jwks::Kid)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("jwks_kid_alg_kty_unique_idx")
          .table(Jwks::Table)
          .col(Jwks::Kid)
          .col(Jwks::Alg)
          .col(Jwks::Kty)
          .unique()
          .to_owned(),
      )
      .await?;

    // Create Clients table
    manager
      .create_table(
        Table::create()
          .table(Clients::Table)
          .if_not_exists()
          .col(pk_auto(Clients::Id))
          .col(integer(Clients::Active).not_null().default(1))
          .col(string(Clients::Name).not_null())
          .col(string(Clients::ClientId).not_null())
          .col(string(Clients::ClientSecret).not_null())
          .col(string(Clients::RedirectUris).null())
          .col(string(Clients::PostLogoutRedirectUris).null())
          .col(string(Clients::LogoUri).null())
          .col(string(Clients::ClientUri).null())
          .col(string(Clients::PolicyUri).null())
          .col(string(Clients::TermsOfServiceUri).null())
          .col(string(Clients::ApplicationType).not_null().default("web"))
          .col(string(Clients::AuthMethod).not_null().default("none"))
          .col(string(Clients::GrantTypes).not_null())
          .col(string(Clients::ResponseTypes).not_null())
          .col(string(Clients::Scopes).not_null())
          .col(string(Clients::Audience).null())
          .col(
            integer(Clients::AccessTokenExpiresInSeconds)
              .not_null()
              .default(3600),
          )
          .col(
            integer(Clients::IdTokenExpiresInSeconds)
              .not_null()
              .default(3600),
          )
          .col(
            integer(Clients::RefreshExpiresInSeconds)
              .not_null()
              .default(604800),
          )
          .col(big_integer(Clients::UpdatedAt).not_null())
          .col(big_integer(Clients::CreatedAt).not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("clients_id_unique_idx")
          .table(Clients::Table)
          .col(Clients::Id)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("clients_client_id_unique_idx")
          .table(Clients::Table)
          .col(Clients::ClientId)
          .unique()
          .to_owned(),
      )
      .await?;

    // Create OAuth2Providers table
    manager
      .create_table(
        Table::create()
          .table(OAuth2Providers::Table)
          .if_not_exists()
          .col(pk_auto(OAuth2Providers::Id))
          .col(integer(OAuth2Providers::Active).not_null().default(1))
          .col(string(OAuth2Providers::Description).not_null())
          .col(string(OAuth2Providers::Uri).not_null())
          .col(string(OAuth2Providers::ClientId).not_null())
          .col(string(OAuth2Providers::ClientSecret).not_null())
          .col(big_integer(OAuth2Providers::UpdatedAt).not_null())
          .col(big_integer(OAuth2Providers::CreatedAt).not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("oauth2_providers_id_unique_idx")
          .table(OAuth2Providers::Table)
          .col(OAuth2Providers::Id)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("oauth2_providers_uri_unique_idx")
          .table(OAuth2Providers::Table)
          .col(OAuth2Providers::Uri)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("oauth2_providers_client_id_idx")
          .table(OAuth2Providers::Table)
          .col(OAuth2Providers::ClientId)
          .to_owned(),
      )
      .await?;

    // Create Applications table
    manager
      .create_table(
        Table::create()
          .table(Applications::Table)
          .if_not_exists()
          .col(pk_auto(Applications::Id))
          .col(integer(Applications::Active).not_null().default(1))
          .col(string(Applications::Uri).not_null())
          .col(string(Applications::Description).not_null())
          .col(string(Applications::RedirectOrigins).not_null())
          .col(big_integer(Applications::UpdatedAt).not_null())
          .col(big_integer(Applications::CreatedAt).not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("applications_id_unique_idx")
          .table(Applications::Table)
          .col(Applications::Id)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("applications_uri_unique_idx")
          .table(Applications::Table)
          .col(Applications::Uri)
          .unique()
          .to_owned(),
      )
      .await?;

    // Create Users table
    manager
      .create_table(
        Table::create()
          .table(Users::Table)
          .if_not_exists()
          .col(pk_auto(Users::Id))
          .col(integer(Users::Active).not_null().default(1))
          .col(string(Users::Username).not_null())
          .col(big_integer(Users::UpdatedAt).not_null())
          .col(big_integer(Users::CreatedAt).not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("users_id_unique_idx")
          .table(Users::Table)
          .col(Users::Id)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("users_username_unique_idx")
          .table(Users::Table)
          .col(Users::Username)
          .unique()
          .to_owned(),
      )
      .await?;

    // Create UserInfos table
    manager
      .create_table(
        Table::create()
          .table(UserInfos::Table)
          .if_not_exists()
          .col(big_integer(UserInfos::UserId))
          .col(string(UserInfos::GivenName).null())
          .col(string(UserInfos::FamilyName).null())
          .col(string(UserInfos::MiddleName).null())
          .col(string(UserInfos::Nickname).null())
          .col(string(UserInfos::ProfilePicture).null())
          .col(string(UserInfos::Website).null())
          .col(string(UserInfos::Gender).null())
          .col(big_integer(UserInfos::Birthdate).null())
          .col(string(UserInfos::ZoneInfo).null())
          .col(string(UserInfos::Locale).null())
          .col(string(UserInfos::Address).null())
          .col(big_integer(UserInfos::UpdatedAt).not_null())
          .col(big_integer(UserInfos::CreatedAt).not_null())
          .primary_key(Index::create().col(UserInfos::UserId))
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_infos_user_id")
              .from(UserInfos::Table, UserInfos::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_infos_user_id_unique_idx")
          .table(UserInfos::Table)
          .col(UserInfos::UserId)
          .unique()
          .to_owned(),
      )
      .await?;

    // Create UserEmails table
    manager
      .create_table(
        Table::create()
          .table(UserEmails::Table)
          .if_not_exists()
          .col(pk_auto(UserEmails::Id))
          .col(integer(UserEmails::UserId).not_null())
          .col(string(UserEmails::Email).not_null())
          .col(integer(UserEmails::Verified).not_null().default(0))
          .col(integer(UserEmails::Primary).not_null().default(0))
          .col(big_integer(UserEmails::UpdatedAt).not_null())
          .col(big_integer(UserEmails::CreatedAt).not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_emails_user_id")
              .from(UserEmails::Table, UserEmails::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_emails_id_unique_idx")
          .table(UserEmails::Table)
          .col(UserEmails::Id)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_emails_email_unique_idx")
          .table(UserEmails::Table)
          .col(UserEmails::Email)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_emails_user_id_idx")
          .table(UserEmails::Table)
          .col(UserEmails::UserId)
          .to_owned(),
      )
      .await?;

    // Create UserPhoneNumbers table
    manager
      .create_table(
        Table::create()
          .table(UserPhoneNumbers::Table)
          .if_not_exists()
          .col(pk_auto(UserPhoneNumbers::Id))
          .col(integer(UserPhoneNumbers::UserId).not_null())
          .col(string(UserPhoneNumbers::PhoneNumber).not_null())
          .col(integer(UserPhoneNumbers::Verified).not_null().default(0))
          .col(integer(UserPhoneNumbers::Primary).not_null().default(0))
          .col(big_integer(UserPhoneNumbers::UpdatedAt).not_null())
          .col(big_integer(UserPhoneNumbers::CreatedAt).not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_phone_numbers_user_id")
              .from(UserPhoneNumbers::Table, UserPhoneNumbers::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_phone_numbers_id_unique_idx")
          .table(UserPhoneNumbers::Table)
          .col(UserPhoneNumbers::Id)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_phone_numbers_phone_number_unique_idx")
          .table(UserPhoneNumbers::Table)
          .col(UserPhoneNumbers::PhoneNumber)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_phone_numbers_user_id_idx")
          .table(UserPhoneNumbers::Table)
          .col(UserPhoneNumbers::UserId)
          .to_owned(),
      )
      .await?;

    // Create UserPasswords table
    manager
      .create_table(
        Table::create()
          .table(UserPasswords::Table)
          .if_not_exists()
          .col(pk_auto(UserPasswords::Id))
          .col(integer(UserPasswords::UserId).not_null())
          .col(integer(UserPasswords::Active).not_null().default(1))
          .col(string(UserPasswords::EncryptedPassword).not_null())
          .col(big_integer(UserPasswords::UpdatedAt).not_null())
          .col(big_integer(UserPasswords::CreatedAt).not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_passwords_user_id")
              .from(UserPasswords::Table, UserPasswords::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_passwords_id_unique_idx")
          .table(UserPasswords::Table)
          .col(UserPasswords::Id)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_passwords_user_id_idx")
          .table(UserPasswords::Table)
          .col(UserPasswords::UserId)
          .to_owned(),
      )
      .await?;

    // Create UserOAuth2Providers table
    manager
      .create_table(
        Table::create()
          .table(UserOAuth2Providers::Table)
          .if_not_exists()
          .col(integer(UserOAuth2Providers::UserId).not_null())
          .col(integer(UserOAuth2Providers::OAuth2ProviderId).not_null())
          .col(string(UserOAuth2Providers::Email).not_null())
          .col(big_integer(UserOAuth2Providers::UpdatedAt).not_null())
          .col(big_integer(UserOAuth2Providers::CreatedAt).not_null())
          .primary_key(
            Index::create()
              .col(UserOAuth2Providers::UserId)
              .col(UserOAuth2Providers::OAuth2ProviderId),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_oauth2_providers_user_id")
              .from(UserOAuth2Providers::Table, UserOAuth2Providers::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_oauth2_providers_oauth2_provider_id")
              .from(
                UserOAuth2Providers::Table,
                UserOAuth2Providers::OAuth2ProviderId,
              )
              .to(OAuth2Providers::Table, OAuth2Providers::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_oauth2_providers_user_id_idx")
          .table(UserOAuth2Providers::Table)
          .col(UserOAuth2Providers::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_oauth2_providers_oauth2_provider_id_idx")
          .table(UserOAuth2Providers::Table)
          .col(UserOAuth2Providers::OAuth2ProviderId)
          .to_owned(),
      )
      .await?;

    // Create UserClients table
    manager
      .create_table(
        Table::create()
          .table(UserClients::Table)
          .if_not_exists()
          .col(integer(UserClients::UserId).not_null())
          .col(string(UserClients::ClientId).not_null())
          .col(string(UserClients::AllowedScopes).not_null())
          .col(big_integer(UserClients::UpdatedAt).not_null())
          .col(big_integer(UserClients::CreatedAt).not_null())
          .primary_key(
            Index::create()
              .col(UserClients::UserId)
              .col(UserClients::ClientId),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_clients_user_id")
              .from(UserClients::Table, UserClients::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_clients_client_id")
              .from(UserClients::Table, UserClients::ClientId)
              .to(Clients::Table, Clients::ClientId)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_clients_user_id_idx")
          .table(UserClients::Table)
          .col(UserClients::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_clients_client_id_idx")
          .table(UserClients::Table)
          .col(UserClients::ClientId)
          .to_owned(),
      )
      .await?;

    // Create Roles table
    manager
      .create_table(
        Table::create()
          .table(Roles::Table)
          .if_not_exists()
          .col(pk_auto(Roles::Id))
          .col(string(Roles::Uri).not_null())
          .col(string(Roles::Description).not_null())
          .col(big_integer(Roles::UpdatedAt).not_null())
          .col(big_integer(Roles::CreatedAt).not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("roles_id_unique_idx")
          .table(Roles::Table)
          .col(Roles::Id)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("roles_uri_unique_idx")
          .table(Roles::Table)
          .col(Roles::Uri)
          .unique()
          .to_owned(),
      )
      .await?;

    // Create Permissions table
    manager
      .create_table(
        Table::create()
          .table(Permissions::Table)
          .if_not_exists()
          .col(pk_auto(Permissions::Id))
          .col(string(Permissions::Uri).not_null())
          .col(string(Permissions::Description).not_null())
          .col(big_integer(Permissions::UpdatedAt).not_null())
          .col(big_integer(Permissions::CreatedAt).not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("permissions_id_unique_idx")
          .table(Permissions::Table)
          .col(Permissions::Id)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("permissions_uri_unique_idx")
          .table(Permissions::Table)
          .col(Permissions::Uri)
          .unique()
          .to_owned(),
      )
      .await?;

    // Create RolesPermissions table
    manager
      .create_table(
        Table::create()
          .table(RolesPermissions::Table)
          .if_not_exists()
          .col(integer(RolesPermissions::RoleId).not_null())
          .col(integer(RolesPermissions::PermissionId).not_null())
          .col(big_integer(RolesPermissions::UpdatedAt).not_null())
          .col(big_integer(RolesPermissions::CreatedAt).not_null())
          .primary_key(
            Index::create()
              .col(RolesPermissions::RoleId)
              .col(RolesPermissions::PermissionId),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_roles_permissions_role_id")
              .from(RolesPermissions::Table, RolesPermissions::RoleId)
              .to(Roles::Table, Roles::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_roles_permissions_permission_id")
              .from(RolesPermissions::Table, RolesPermissions::PermissionId)
              .to(Permissions::Table, Permissions::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("roles_permissions_role_id_permission_id_unique_idx")
          .table(RolesPermissions::Table)
          .col(RolesPermissions::RoleId)
          .col(RolesPermissions::PermissionId)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("roles_permissions_role_id_idx")
          .table(RolesPermissions::Table)
          .col(RolesPermissions::RoleId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("roles_permissions_permission_id_idx")
          .table(RolesPermissions::Table)
          .col(RolesPermissions::PermissionId)
          .to_owned(),
      )
      .await?;

    // Create UserRoles table
    manager
      .create_table(
        Table::create()
          .table(UserRoles::Table)
          .if_not_exists()
          .col(integer(UserRoles::UserId).not_null())
          .col(integer(UserRoles::RoleId).not_null())
          .col(big_integer(UserRoles::UpdatedAt).not_null())
          .col(big_integer(UserRoles::CreatedAt).not_null())
          .primary_key(
            Index::create()
              .col(UserRoles::UserId)
              .col(UserRoles::RoleId),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_roles_user_id")
              .from(UserRoles::Table, UserRoles::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk_user_roles_role_id")
              .from(UserRoles::Table, UserRoles::RoleId)
              .to(Roles::Table, Roles::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_roles_user_id_role_id_unique_idx")
          .table(UserRoles::Table)
          .col(UserRoles::UserId)
          .col(UserRoles::RoleId)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_roles_user_id_idx")
          .table(UserRoles::Table)
          .col(UserRoles::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("user_roles_role_id_idx")
          .table(UserRoles::Table)
          .col(UserRoles::RoleId)
          .to_owned(),
      )
      .await?;

    // Create KeyValues table
    manager
      .create_table(
        Table::create()
          .table(KeyValues::Table)
          .if_not_exists()
          .col(string(KeyValues::Key))
          .col(string(KeyValues::Value).not_null())
          .col(big_integer(KeyValues::ExpiresAt).null())
          .col(big_integer(KeyValues::UpdatedAt).not_null())
          .col(big_integer(KeyValues::CreatedAt).not_null())
          .primary_key(Index::create().col(KeyValues::Key))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("key_values_key_unique_idx")
          .table(KeyValues::Table)
          .col(KeyValues::Key)
          .unique()
          .to_owned(),
      )
      .await?;

    // Create RevokedTokens table
    manager
      .create_table(
        Table::create()
          .table(RevokedTokens::Table)
          .if_not_exists()
          .col(string(RevokedTokens::Token))
          .col(big_integer(RevokedTokens::ExpiresAt).not_null())
          .col(big_integer(RevokedTokens::CreatedAt).not_null())
          .primary_key(Index::create().col(RevokedTokens::Token))
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("revoked_tokens_token_unique_idx")
          .table(RevokedTokens::Table)
          .col(RevokedTokens::Token)
          .unique()
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("revoked_tokens_expires_at_idx")
          .table(RevokedTokens::Table)
          .col(RevokedTokens::ExpiresAt)
          .to_owned(),
      )
      .await?;

    // Insert seed data
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis() as i64;

    manager
      .exec_stmt(
        Query::insert()
          .into_table(Roles::Table)
          .columns(vec![
            Roles::Uri,
            Roles::Description,
            Roles::UpdatedAt,
            Roles::CreatedAt,
          ])
          .values_panic(vec![
            "admin".into(),
            "Administrator role".into(),
            now.into(),
            now.into(),
          ])
          .to_owned(),
      )
      .await?;

    manager
      .exec_stmt(
        Query::insert()
          .into_table(Permissions::Table)
          .columns(vec![
            Permissions::Uri,
            Permissions::Description,
            Permissions::UpdatedAt,
            Permissions::CreatedAt,
          ])
          .values_panic(vec![
            "admin:*".into(),
            "Administer all resources".into(),
            now.into(),
            now.into(),
          ])
          .to_owned(),
      )
      .await?;

    manager
      .exec_stmt(
        Query::insert()
          .into_table(RolesPermissions::Table)
          .columns(vec![
            RolesPermissions::RoleId,
            RolesPermissions::PermissionId,
            RolesPermissions::UpdatedAt,
            RolesPermissions::CreatedAt,
          ])
          .values_panic(vec![1i32.into(), 1i32.into(), now.into(), now.into()])
          .to_owned(),
      )
      .await?;

    manager
      .exec_stmt(
        Query::insert()
          .into_table(Users::Table)
          .columns(vec![Users::Username, Users::UpdatedAt, Users::CreatedAt])
          .values_panic(vec!["admin".into(), now.into(), now.into()])
          .to_owned(),
      )
      .await?;

    manager
      .exec_stmt(
        Query::insert()
          .into_table(UserInfos::Table)
          .columns(vec![
            UserInfos::UserId,
            UserInfos::Nickname,
            UserInfos::UpdatedAt,
            UserInfos::CreatedAt,
          ])
          .values_panic(vec![1i32.into(), "admin".into(), now.into(), now.into()])
          .to_owned(),
      )
      .await?;

    manager
      .exec_stmt(
        Query::insert()
          .into_table(UserRoles::Table)
          .columns(vec![
            UserRoles::RoleId,
            UserRoles::UserId,
            UserRoles::UpdatedAt,
            UserRoles::CreatedAt,
          ])
          .values_panic(vec![1i32.into(), 1i32.into(), now.into(), now.into()])
          .to_owned(),
      )
      .await?;

    manager
            .exec_stmt(
                Query::insert()
                    .into_table(UserPasswords::Table)
                    .columns(vec![UserPasswords::UserId, UserPasswords::EncryptedPassword, UserPasswords::UpdatedAt, UserPasswords::CreatedAt])
                    .values_panic(vec![
                        1i32.into(),
                        "$argon2id$v=19$m=19,t=2,p=1$cmc5ZXVXT1N0RmxjZFR1NQ$/0nLLEJDUFjP/lO6UhUHlzvL6Zlz1NO8BW+XdMNTG3c".into(),
                        now.into(),
                        now.into(),
                    ])
                    .to_owned(),
            )
            .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(UserRoles::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(RolesPermissions::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(Permissions::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(Roles::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(UserClients::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(UserOAuth2Providers::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(UserPasswords::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(UserPhoneNumbers::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(UserEmails::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(UserInfos::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(Users::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(Applications::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(OAuth2Providers::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(Clients::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(Jwks::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(KeyValues::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(RevokedTokens::Table).to_owned())
      .await?;

    Ok(())
  }
}

// Enum definitions for table names and columns
#[derive(Iden)]
enum Jwks {
  Table,
  Kid,
  Active,
  Kty,
  Alg,
  #[iden = "use"]
  Use,
  KeyOps,
  N,
  E,
  D,
  P,
  Q,
  Dp,
  Dq,
  Qi,
  Crv,
  X,
  Y,
  #[iden = "d_ec"]
  DEc,
  K,
  X5u,
  X5c,
  X5t,
  #[iden = "x5t_s256"]
  X5tS256,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum Clients {
  Table,
  Id,
  Active,
  Name,
  ClientId,
  ClientSecret,
  RedirectUris,
  PostLogoutRedirectUris,
  LogoUri,
  ClientUri,
  PolicyUri,
  TermsOfServiceUri,
  ApplicationType,
  AuthMethod,
  GrantTypes,
  ResponseTypes,
  Scopes,
  Audience,
  AccessTokenExpiresInSeconds,
  IdTokenExpiresInSeconds,
  RefreshExpiresInSeconds,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum OAuth2Providers {
  Table,
  Id,
  Active,
  Description,
  Uri,
  ClientId,
  ClientSecret,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum Applications {
  Table,
  Id,
  Active,
  Uri,
  Description,
  RedirectOrigins,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum Users {
  Table,
  Id,
  Active,
  Username,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum UserInfos {
  Table,
  UserId,
  GivenName,
  FamilyName,
  MiddleName,
  Nickname,
  ProfilePicture,
  Website,
  Gender,
  Birthdate,
  ZoneInfo,
  Locale,
  Address,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum UserEmails {
  Table,
  Id,
  UserId,
  Email,
  Verified,
  Primary,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum UserPhoneNumbers {
  Table,
  Id,
  UserId,
  PhoneNumber,
  Verified,
  Primary,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum UserPasswords {
  Table,
  Id,
  UserId,
  Active,
  EncryptedPassword,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum UserOAuth2Providers {
  Table,
  UserId,
  OAuth2ProviderId,
  Email,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum UserClients {
  Table,
  UserId,
  ClientId,
  AllowedScopes,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum Roles {
  Table,
  Id,
  Uri,
  Description,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum Permissions {
  Table,
  Id,
  Uri,
  Description,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum RolesPermissions {
  Table,
  RoleId,
  PermissionId,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum UserRoles {
  Table,
  UserId,
  RoleId,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum KeyValues {
  Table,
  Key,
  Value,
  ExpiresAt,
  UpdatedAt,
  CreatedAt,
}

#[derive(Iden)]
enum RevokedTokens {
  Table,
  Token,
  ExpiresAt,
  CreatedAt,
}
