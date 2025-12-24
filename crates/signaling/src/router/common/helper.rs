use crate::{config::AppConfig, router::common::entity::Claims};

pub fn parse_jwt<T>(
  jwt: &str,
  app_config: &AppConfig,
  decoding_key: jsonwebtoken::DecodingKey,
  algorithm: jsonwebtoken::Algorithm,
) -> Result<jsonwebtoken::TokenData<T>, jsonwebtoken::errors::Error>
where
  T: Claims,
{
  let mut validation = jsonwebtoken::Validation::new(algorithm);
  validation.validate_nbf = true;
  validation.validate_aud = false;
  validation.set_issuer(&[app_config.url()]);

  jsonwebtoken::decode(jwt, &decoding_key, &validation)
}

pub fn to_public_jwk(jwk: &jsonwebtoken::jwk::Jwk) -> jsonwebtoken::jwk::Jwk {
  let mut public_jwk = jwk.clone();
  public_jwk.common.key_operations = public_jwk.common.key_operations.map(|key_operations| {
    key_operations
      .into_iter()
      .filter(is_public_key_operation)
      .collect()
  });
  public_jwk
}

pub fn is_public_key_operation(key_operation: &jsonwebtoken::jwk::KeyOperations) -> bool {
  matches!(
    key_operation,
    jsonwebtoken::jwk::KeyOperations::Verify
      | jsonwebtoken::jwk::KeyOperations::Encrypt
      | jsonwebtoken::jwk::KeyOperations::WrapKey
  )
}
