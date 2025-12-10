// Re-export ORM types and functions for backward compatibility
pub use super::orm::{
  // Type aliases
  UserClientModel as UserClientSQLRow,
  UserEmailModel as UserEmailSQLRow,
  UserEmailModelExt,
  UserInfoModel as UserInfoSQLRow,
  UserModel as UserSQLRow,
  UserModelExt,
  UserOAuth2ProviderModel as UserOAuth2ProviderSQLRow,
  UserPasswordModel as UserPasswordSQLRow,
  UserPhoneNumberModel as UserPhoneNumberSQLRow,
  UserPhoneNumberModelExt,
  // Functions
  assign_user_role,
  create_user,
  create_user_email,
  create_user_phone_number,
  create_user_with_email_and_password,
  create_user_with_password,
  delete_user,
  delete_user_email,
  delete_user_oauth2_provider,
  delete_user_phone_number,
  get_user_active_password_by_user_id,
  get_user_by_id,
  get_user_by_username,
  get_user_by_username_or_primary_email,
  get_user_client_by_client_id,
  get_user_email_by_id,
  get_user_info_by_user_id,
  get_user_oauth2_provider_by_id,
  get_user_oauth2_providers,
  get_user_permissions,
  get_user_phone_number_by_id,
  get_user_role_permissions_by_user_id,
  get_user_roles_by_user_id,
  link_user_oauth2_provider,
  list_user_emails_by_user_id,
  list_user_phone_numbers_by_user_id,
  list_users,
  remove_user_role,
  update_user,
  update_user_email_primary,
  update_user_info,
  update_user_password,
  update_user_phone_number_primary,
  upsert_user_client,
  verify_user_email,
  verify_user_phone_number,
};

// Need to add these functions to ORM
use os_model::entities::{prelude::*, *};
use sea_orm::*;

pub async fn get_user_primary_email(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<user_emails::Model>, DbErr> {
  UserEmails::find()
    .filter(user_emails::Column::UserId.eq(user_id))
    .filter(user_emails::Column::Primary.eq(1))
    .one(db)
    .await
}

pub async fn get_user_primary_phone_number(
  db: &DatabaseConnection,
  user_id: i64,
) -> Result<Option<user_phone_numbers::Model>, DbErr> {
  UserPhoneNumbers::find()
    .filter(user_phone_numbers::Column::UserId.eq(user_id))
    .filter(user_phone_numbers::Column::Primary.eq(1))
    .one(db)
    .await
}
