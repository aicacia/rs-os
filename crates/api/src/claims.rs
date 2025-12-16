use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Default, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct BasicClaims {
  pub r#type: String,
  pub exp: i64,
  pub iat: i64,
  pub nbf: i64,
  pub iss: String,
  pub client: String,
  pub aud: String,
  pub sub: String,
  pub scope: String,
}

pub trait Claims: Serialize + Send + Sync + DeserializeOwned {
  fn r#type(&self) -> &str;
  fn exp(&self) -> i64;
  fn iat(&self) -> i64;
  fn nbf(&self) -> i64;
  fn iss(&self) -> &str;
  fn client(&self) -> &str;
  fn aud(&self) -> &str;
  fn sub(&self) -> &str;
  fn scope(&self) -> &str;

  fn has_scope(&self, scope: &str) -> bool {
    self.scope().split_whitespace().any(|s| s == scope)
  }
}

impl Claims for BasicClaims {
  fn r#type(&self) -> &str {
    &self.r#type
  }
  fn exp(&self) -> i64 {
    self.exp
  }
  fn iat(&self) -> i64 {
    self.iat
  }
  fn nbf(&self) -> i64 {
    self.nbf
  }
  fn iss(&self) -> &str {
    &self.iss
  }
  fn client(&self) -> &str {
    &self.client
  }
  fn aud(&self) -> &str {
    &self.aud
  }
  fn sub(&self) -> &str {
    &self.sub
  }
  fn scope(&self) -> &str {
    &self.scope
  }
}
