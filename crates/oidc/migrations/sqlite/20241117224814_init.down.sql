PRAGMA foreign_keys = OFF;

DROP TABLE IF EXISTS "user_oauth2_providers";
DROP TABLE IF EXISTS "user_passwords";
DROP TABLE IF EXISTS "user_phone_numbers";
DROP TABLE IF EXISTS "user_emails";
DROP TABLE IF EXISTS "user_infos";

DROP TABLE IF EXISTS "users";
DROP TABLE IF EXISTS "applications";
DROP TABLE IF EXISTS "oauth2_providers";
DROP TABLE IF EXISTS "clients";
DROP TABLE IF EXISTS "jwks";
DROP TABLE IF EXISTS "configs";
DROP TABLE IF EXISTS "key_values";

DROP INDEX IF EXISTS "configs_key_unique_idx";

DROP INDEX IF EXISTS "jwks_kid_unique_idx";
DROP INDEX IF EXISTS "jwks_kid_alg_kty_unique_idx";

DROP INDEX IF EXISTS "clients_id_unique_idx";
DROP INDEX IF EXISTS "clients_client_id_unique_idx";

DROP INDEX IF EXISTS "oauth2_providers_id_unique_idx";
DROP INDEX IF EXISTS "oauth2_providers_uri_unique_idx";
DROP INDEX IF EXISTS "oauth2_providers_name_unique_idx";
DROP INDEX IF EXISTS "oauth2_providers_client_id_idx";

DROP INDEX IF EXISTS "applications_id_unique_idx";
DROP INDEX IF EXISTS "applications_uri_unique_idx";
DROP INDEX IF EXISTS "applications_name_unique_idx";

DROP INDEX IF EXISTS "users_id_unique_idx";
DROP INDEX IF EXISTS "users_username_unique_idx";

DROP INDEX IF EXISTS "user_infos_user_id_unique_idx";

DROP INDEX IF EXISTS "user_emails_id_unique_idx";
DROP INDEX IF EXISTS "user_emails_email_unique_idx";
DROP INDEX IF EXISTS "user_emails_user_id_idx";

DROP INDEX IF EXISTS "user_phone_numbers_id_unique_idx";
DROP INDEX IF EXISTS "user_phone_numbers_phone_number_unique_idx";
DROP INDEX IF EXISTS "user_phone_numbers_user_id_idx";

DROP INDEX IF EXISTS "user_passwords_id_unique_idx";
DROP INDEX IF EXISTS "user_passwords_user_id_idx";

DROP INDEX IF EXISTS "user_oauth2_providers_user_id_idx";
DROP INDEX IF EXISTS "user_oauth2_providers_oauth2_provider_id_idx";

DROP INDEX IF EXISTS "key_values_key_unique_idx";

PRAGMA foreign_keys = ON;
