CREATE TABLE jwks (
  "kid" SERIAL PRIMARY KEY,
  "active" INTEGER NOT NULL DEFAULT 1,

  "kty" TEXT NOT NULL,           -- Key type: "RSA", "EC", "oct", "OKP"
  "alg" TEXT NOT NULL,           -- Algorithm (e.g., RS256, ES256, HS256)
  "use" TEXT,                    -- "sig", "enc", etc.
  "key_ops" TEXT,                -- JSON array of key operations

  -- RSA fields
  "n" TEXT,                      -- Modulus
  "e" TEXT,                      -- Exponent
  "d" TEXT,                      -- Private exponent (optional)
  "p" TEXT,                      -- First prime factor
  "q" TEXT,                      -- Second prime factor
  "dp" TEXT,                     -- First factor CRT exponent
  "dq" TEXT,                     -- Second factor CRT exponent
  "qi" TEXT,                     -- First CRT coefficient

  -- EC fields
  "crv" TEXT,                    -- Curve name (e.g., P-256, secp256k1)
  "x" TEXT,                      -- X coordinate
  "y" TEXT,                      -- Y coordinate
  "d_ec" TEXT,                   -- Private key for EC

  -- Symmetric (oct) fields
  "k" TEXT,                      -- Symmetric key value

  -- X.509 fields
  "x5u" TEXT,                    -- URL to X.509 public key
  "x5c" TEXT,                    -- X.509 certificate chain (base64 DER, array)
  "x5t" TEXT,                    -- X.509 SHA-1 thumbprint
  "x5t_s256" TEXT,               -- X.509 SHA-256 thumbprint

	"updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
	"created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT
);
CREATE UNIQUE INDEX "jwks_kid_unique_idx" ON "jwks" ("kid");
CREATE UNIQUE INDEX "jwks_kid_alg_kty_unique_idx" ON "jwks" ("kid", "alg", "kty");


CREATE TABLE "clients" (
  "id" SERIAL PRIMARY KEY,
  "active" INTEGER NOT NULL DEFAULT 1,
  "name" TEXT NOT NULL,
	"client_id" TEXT NOT NULL,
  "client_secret" TEXT NOT NULL,
  "redirect_uris" TEXT,
  "post_logout_redirect_uris" TEXT,
  "logo_uri" TEXT,
  "client_uri" TEXT,
  "policy_uri" TEXT,
  "terms_of_service_uri" TEXT,
  "application_type" TEXT NOT NULL DEFAULT 'web',
  "auth_method" TEXT NOT NULL DEFAULT 'none',
  "grant_types" TEXT NOT NULL,
  "response_types" TEXT NOT NULL,
  "scopes" TEXT NOT NULL,
  "audience" TEXT,
	"access_token_expires_in_seconds" INTEGER NOT NULL DEFAULT 3600,
	"id_token_expires_in_seconds" INTEGER NOT NULL DEFAULT 3600,
	"refresh_expires_in_seconds" INTEGER NOT NULL DEFAULT 604800,
	"updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
	"created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT
);
CREATE UNIQUE INDEX "clients_id_unique_idx" ON "clients" ("id");
CREATE UNIQUE INDEX "clients_client_id_unique_idx" ON "clients" ("client_id");


CREATE TABLE "oauth2_providers" (
  "id" SERIAL PRIMARY KEY,
  "active" INTEGER NOT NULL DEFAULT 1,
  "description" TEXT NOT NULL,
  "uri" TEXT NOT NULL,
  "client_id" TEXT NOT NULL,
  "client_secret" TEXT NOT NULL,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT
);
CREATE UNIQUE INDEX "oauth2_providers_id_unique_idx" ON "oauth2_providers" ("id");
CREATE UNIQUE INDEX "oauth2_providers_uri_unique_idx" ON "oauth2_providers" ("uri");
CREATE INDEX "oauth2_providers_client_id_idx" ON "oauth2_providers" ("client_id");


CREATE TABLE "applications" (
  "id" SERIAL PRIMARY KEY,
  "active" INTEGER NOT NULL DEFAULT 1,
  "uri" TEXT NOT NULL,
  "description" TEXT NOT NULL,
  "redirect_origins" TEXT NOT NULL,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT
);
CREATE UNIQUE INDEX "applications_id_unique_idx" ON "applications" ("id");
CREATE UNIQUE INDEX "applications_uri_unique_idx" ON "applications" ("uri");


CREATE TABLE "users" (
  "id" SERIAL PRIMARY KEY,
  "active" INTEGER NOT NULL DEFAULT 1,
  "username" TEXT NOT NULL,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT
);
CREATE UNIQUE INDEX "users_id_unique_idx" ON "users" ("id");
CREATE UNIQUE INDEX "users_username_unique_idx" ON "users" ("username");


CREATE TABLE "user_infos"(
	"user_id" INTEGER NOT NULL PRIMARY KEY,
	"name" TEXT,
	"given_name" TEXT,
	"family_name" TEXT,
	"middle_name" TEXT,
	"nickname" TEXT,
	"profile_picture" TEXT,
	"website" TEXT,
	"gender" TEXT,
	"birthdate" BIGINT,
	"zone_info" TEXT,
	"locale" TEXT,
	"address" TEXT,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
CREATE UNIQUE INDEX "user_infos_user_id_unique_idx" ON "user_infos" ("user_id");


CREATE TABLE "user_emails" (
  "id" SERIAL PRIMARY KEY,
  "user_id" INTEGER NOT NULL,
  "email" TEXT NOT NULL,
  "verified" INTEGER NOT NULL DEFAULT 0,
  "primary" INTEGER NOT NULL DEFAULT 0,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
CREATE UNIQUE INDEX "user_emails_id_unique_idx" ON "user_emails" ("id");
CREATE UNIQUE INDEX "user_emails_email_unique_idx" ON "user_emails" ("email");
CREATE INDEX "user_emails_user_id_idx" ON "user_emails" ("user_id");


CREATE TABLE "user_phone_numbers" (
  "id" SERIAL PRIMARY KEY,
  "user_id" INTEGER NOT NULL,
  "phone_number" TEXT NOT NULL,
  "verified" INTEGER NOT NULL DEFAULT 0,
  "primary" INTEGER NOT NULL DEFAULT 0,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
CREATE UNIQUE INDEX "user_phone_numbers_id_unique_idx" ON "user_phone_numbers" ("id");
CREATE UNIQUE INDEX "user_phone_numbers_phone_number_unique_idx" ON "user_phone_numbers" ("phone_number");
CREATE INDEX "user_phone_numbers_user_id_idx" ON "user_phone_numbers" ("user_id");


CREATE TABLE "user_passwords" (
  "id" SERIAL PRIMARY KEY,
  "user_id" INTEGER NOT NULL,
  "active" INTEGER NOT NULL DEFAULT 1,
  "encrypted_password" TEXT NOT NULL,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
CREATE UNIQUE INDEX "user_passwords_id_unique_idx" ON "user_passwords" ("id");
CREATE INDEX "user_passwords_user_id_idx" ON "user_passwords" ("user_id");


CREATE TABLE "user_oauth2_providers" (
  "user_id" INTEGER NOT NULL,
  "oauth2_provider_id" INTEGER NOT NULL,
  "email" TEXT NOT NULL,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  PRIMARY KEY ("user_id", "oauth2_provider_id"),
  FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE,
  FOREIGN KEY ("oauth2_provider_id") REFERENCES "oauth2_providers" ("id") ON DELETE CASCADE
);
CREATE INDEX "user_oauth2_providers_user_id_idx" ON "user_oauth2_providers" ("user_id");
CREATE INDEX "user_oauth2_providers_oauth2_provider_id_idx" ON "user_oauth2_providers" ("oauth2_provider_id");


CREATE TABLE "roles" (
  "id" SERIAL PRIMARY KEY,
  "uri" TEXT NOT NULL,
  "description" TEXT NOT NULL,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT
);
CREATE UNIQUE INDEX "roles_id_unique_idx" ON "roles" ("id");
CREATE UNIQUE INDEX "roles_uri_unique_idx" ON "roles" ("uri");


CREATE TABLE "permissions" (
  "id" SERIAL PRIMARY KEY,
  "uri" TEXT NOT NULL,
  "description" TEXT NOT NULL,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT
);
CREATE UNIQUE INDEX "permissions_id_unique_idx" ON "permissions" ("id");
CREATE UNIQUE INDEX "permissions_uri_unique_idx" ON "permissions" ("uri");


CREATE TABLE "roles_permissions" (
  "role_id" INTEGER NOT NULL,
  "permission_id" INTEGER NOT NULL,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  FOREIGN KEY ("role_id") REFERENCES "roles" ("id") ON DELETE CASCADE,
  FOREIGN KEY ("permission_id") REFERENCES "permissions" ("id") ON DELETE CASCADE
);
CREATE UNIQUE INDEX "roles_permissions_role_id_permission_id_unique_idx" ON "roles_permissions" ("role_id", "permission_id");
CREATE INDEX "roles_permissions_role_id_idx" ON "roles_permissions" ("role_id");
CREATE INDEX "roles_permissions_permission_id_idx" ON "roles_permissions" ("permission_id");


CREATE TABLE "user_roles" (
  "user_id" INTEGER NOT NULL,
  "role_id" INTEGER NOT NULL,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE,
  FOREIGN KEY ("role_id") REFERENCES "roles" ("id") ON DELETE CASCADE
);
CREATE UNIQUE INDEX "user_roles_user_id_role_id_unique_idx" ON "user_roles" ("user_id", "role_id");
CREATE INDEX "user_roles_user_id_idx" ON "user_roles" ("user_id");
CREATE INDEX "user_roles_role_id_idx" ON "user_roles" ("role_id");



CREATE TABLE "key_values" (
  "key" TEXT NOT NULL PRIMARY KEY,
  "value" TEXT NOT NULL,
  "expires_at" BIGINT,
  "updated_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT,
  "created_at" BIGINT NOT NULL DEFAULT FLOOR(EXTRACT(EPOCH FROM CURRENT_TIMESTAMP))::BIGINT
);
CREATE UNIQUE INDEX "key_values_key_unique_idx" ON "key_values" ("key");