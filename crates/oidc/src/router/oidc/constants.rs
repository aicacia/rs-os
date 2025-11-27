pub const TAG: &str = "oidc";
pub const DESCRIPTION: &str = "OpenID Connect endpoints";

pub const GRANT_TYPE_PASSWORD: &str = "password";
pub const GRANT_TYPE_AUTHORIZATION_CODE: &str = "authorization_code";
pub const GRANT_TYPE_REFRESH_TOKEN: &str = "refresh_token";
pub const GRANT_TYPE_REVOKE: &str = "revoke";

pub const ALWAYS_ALLOWED_GRANT_TYPES: &[&str] = &[GRANT_TYPE_REVOKE];
