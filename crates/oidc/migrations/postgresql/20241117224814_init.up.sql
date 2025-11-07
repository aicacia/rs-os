CREATE TABLE configs (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX configs_key_unique_idx ON configs (key);


CREATE TABLE jwks (
  kid BIGSERIAL PRIMARY KEY,
  active BOOLEAN NOT NULL DEFAULT TRUE,

  kty TEXT NOT NULL,            -- Key type: "RSA", "EC", "oct", "OKP"
  alg TEXT NOT NULL,            -- Algorithm
  use TEXT,                     -- "sig", "enc", etc.
  key_ops JSONB DEFAULT NULL,   -- JSON array of key operations

  -- RSA fields
  n TEXT,
  e TEXT,
  d TEXT,
  p TEXT,
  q TEXT,
  dp TEXT,
  dq TEXT,
  qi TEXT,

  -- EC fields
  crv TEXT,
  x TEXT,
  y TEXT,
  d_ec TEXT,

  -- Symmetric (oct) fields
  k TEXT,

  -- X.509 fields
  x5u TEXT,
  x5c JSONB,                    -- certificate chain (array)
  x5t TEXT,
  x5t_s256 TEXT,

  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX jwks_kid_unique_idx ON jwks (kid);
CREATE UNIQUE INDEX jwks_kid_alg_kty_unique_idx ON jwks (kid, alg, kty);


CREATE TABLE clients (
  id BIGSERIAL PRIMARY KEY,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  name TEXT NOT NULL,
  client_id TEXT NOT NULL,
  client_secret TEXT NOT NULL,
  redirect_uris TEXT[],
  post_logout_redirect_uris TEXT[],
  logo_uri TEXT,
  client_uri TEXT,
  policy_uri TEXT,
  terms_of_service_uri TEXT,
  application_type TEXT NOT NULL DEFAULT 'web',
  auth_method TEXT NOT NULL DEFAULT 'none',
  grant_types TEXT[] NOT NULL,
  response_types TEXT[] NOT NULL,
  scopes TEXT[] NOT NULL,
  audience TEXT[],
  access_token_expires_in_seconds INTEGER NOT NULL DEFAULT 3600,
  id_token_expires_in_seconds INTEGER NOT NULL DEFAULT 3600,
  refresh_expires_in_seconds INTEGER NOT NULL DEFAULT 604800,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX clients_id_unique_idx ON clients (id);
CREATE UNIQUE INDEX clients_client_id_unique_idx ON clients (client_id);


CREATE TABLE oauth2_providers (
  id BIGSERIAL PRIMARY KEY,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  name TEXT NOT NULL,
  uri TEXT NOT NULL,
  client_id TEXT NOT NULL,
  client_secret TEXT NOT NULL,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX oauth2_providers_id_unique_idx ON oauth2_providers (id);
CREATE UNIQUE INDEX oauth2_providers_uri_unique_idx ON oauth2_providers (uri);
CREATE UNIQUE INDEX oauth2_providers_name_unique_idx ON oauth2_providers (name);
CREATE INDEX oauth2_providers_client_id_idx ON oauth2_providers (client_id);


CREATE TABLE applications (
  id BIGSERIAL PRIMARY KEY,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  uri TEXT NOT NULL,
  name TEXT NOT NULL,
  redirect_origins TEXT[] NOT NULL,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX applications_id_unique_idx ON applications (id);
CREATE UNIQUE INDEX applications_uri_unique_idx ON applications (uri);
CREATE UNIQUE INDEX applications_name_unique_idx ON applications (name);


CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  username TEXT NOT NULL,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX users_id_unique_idx ON users (id);
CREATE UNIQUE INDEX users_username_unique_idx ON users (username);


CREATE TABLE user_infos (
  user_id BIGINT PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
  name TEXT,
  given_name TEXT,
  family_name TEXT,
  middle_name TEXT,
  nickname TEXT,
  profile_picture TEXT,
  website TEXT,
  gender TEXT,
  birthdate BIGINT,
  zone_info TEXT,
  locale TEXT,
  address JSONB,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX user_infos_user_id_unique_idx ON user_infos (user_id);


CREATE TABLE user_emails (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  email TEXT NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT FALSE,
  primary_email BOOLEAN NOT NULL DEFAULT FALSE,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX user_emails_id_unique_idx ON user_emails (id);
CREATE UNIQUE INDEX user_emails_email_unique_idx ON user_emails (email);
CREATE INDEX user_emails_user_id_idx ON user_emails (user_id);


CREATE TABLE user_phone_numbers (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  phone_number TEXT NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT FALSE,
  primary_phone BOOLEAN NOT NULL DEFAULT FALSE,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX user_phone_numbers_id_unique_idx ON user_phone_numbers (id);
CREATE UNIQUE INDEX user_phone_numbers_phone_number_unique_idx ON user_phone_numbers (phone_number);
CREATE INDEX user_phone_numbers_user_id_idx ON user_phone_numbers (user_id);


CREATE TABLE user_passwords (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  encrypted_password TEXT NOT NULL,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX user_passwords_id_unique_idx ON user_passwords (id);
CREATE INDEX user_passwords_user_id_idx ON user_passwords (user_id);


CREATE TABLE user_oauth2_providers (
  user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  oauth2_provider_id BIGINT NOT NULL REFERENCES oauth2_providers (id) ON DELETE CASCADE,
  email TEXT NOT NULL,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  PRIMARY KEY (user_id, oauth2_provider_id)
);
CREATE INDEX user_oauth2_providers_user_id_idx ON user_oauth2_providers (user_id);
CREATE INDEX user_oauth2_providers_oauth2_provider_id_idx ON user_oauth2_providers (oauth2_provider_id);


CREATE TABLE key_values (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  expires_at BIGINT,
  updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint),
  created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM now())::bigint)
);
CREATE UNIQUE INDEX key_values_key_unique_idx ON key_values (key);
