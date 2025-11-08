-- key_values
DROP INDEX IF EXISTS "key_values_key_unique_idx";
DROP TABLE IF EXISTS "key_values";

-- user_roles
DROP INDEX IF EXISTS "user_roles_role_id_idx";
DROP INDEX IF EXISTS "user_roles_user_id_idx";
DROP INDEX IF EXISTS "user_roles_user_id_role_id_unique_idx";
DROP TABLE IF EXISTS "user_roles";

-- roles_permissions
DROP INDEX IF EXISTS "roles_permissions_permission_id_idx";
DROP INDEX IF EXISTS "roles_permissions_role_id_idx";
DROP INDEX IF EXISTS "roles_permissions_role_id_permission_id_unique_idx";
DROP TABLE IF EXISTS "roles_permissions";

-- permissions
DROP INDEX IF EXISTS "permissions_name_unique_idx";
DROP INDEX IF EXISTS "permissions_uri_unique_idx";
DROP INDEX IF EXISTS "permissions_id_unique_idx";
DROP TABLE IF EXISTS "permissions";

-- roles
DROP INDEX IF EXISTS "roles_name_unique_idx";
DROP INDEX IF EXISTS "roles_uri_unique_idx";
DROP INDEX IF EXISTS "roles_id_unique_idx";
DROP TABLE IF EXISTS "roles";

-- user_oauth2_providers
DROP INDEX IF EXISTS "user_oauth2_providers_oauth2_provider_id_idx";
DROP INDEX IF EXISTS "user_oauth2_providers_user_id_idx";
DROP TABLE IF EXISTS "user_oauth2_providers";

-- user_passwords
DROP INDEX IF EXISTS "user_passwords_user_id_idx";
DROP INDEX IF EXISTS "user_passwords_id_unique_idx";
DROP TABLE IF EXISTS "user_passwords";

-- user_phone_numbers
DROP INDEX IF EXISTS "user_phone_numbers_user_id_idx";
DROP INDEX IF EXISTS "user_phone_numbers_phone_number_unique_idx";
DROP INDEX IF EXISTS "user_phone_numbers_id_unique_idx";
DROP TABLE IF EXISTS "user_phone_numbers";

-- user_emails
DROP INDEX IF EXISTS "user_emails_user_id_idx";
DROP INDEX IF EXISTS "user_emails_email_unique_idx";
DROP INDEX IF EXISTS "user_emails_id_unique_idx";
DROP TABLE IF EXISTS "user_emails";

-- user_infos
DROP INDEX IF EXISTS "user_infos_user_id_unique_idx";
DROP TABLE IF EXISTS "user_infos";

-- users
DROP INDEX IF EXISTS "users_username_unique_idx";
DROP INDEX IF EXISTS "users_id_unique_idx";
DROP TABLE IF EXISTS "users";

-- applications
DROP INDEX IF EXISTS "applications_name_unique_idx";
DROP INDEX IF EXISTS "applications_uri_unique_idx";
DROP INDEX IF EXISTS "applications_id_unique_idx";
DROP TABLE IF EXISTS "applications";

-- oauth2_providers
DROP INDEX IF EXISTS "oauth2_providers_client_id_idx";
DROP INDEX IF EXISTS "oauth2_providers_name_unique_idx";
DROP INDEX IF EXISTS "oauth2_providers_uri_unique_idx";
DROP INDEX IF EXISTS "oauth2_providers_id_unique_idx";
DROP TABLE IF EXISTS "oauth2_providers";

-- clients
DROP INDEX IF EXISTS "clients_client_id_unique_idx";
DROP INDEX IF EXISTS "clients_id_unique_idx";
DROP TABLE IF EXISTS "clients";

-- jwks
DROP INDEX IF EXISTS "jwks_kid_alg_kty_unique_idx";
DROP INDEX IF EXISTS "jwks_kid_unique_idx";
DROP TABLE IF EXISTS "jwks";

-- configs
DROP INDEX IF EXISTS "configs_key_unique_idx";
DROP TABLE IF EXISTS "configs";