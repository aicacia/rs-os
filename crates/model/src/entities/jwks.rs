use sea_orm::entity::prelude::*;

#[derive(Clone, Default, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jwks")]
pub struct Model {
  #[sea_orm(primary_key, unique)]
  pub kid: i64,
  #[sea_orm(default_value = "1")]
  pub active: i64,
  #[sea_orm(unique_key = "jwks_kid_alg_kty_unique_idx")]
  pub kty: String,
  #[sea_orm(unique_key = "jwks_kid_alg_kty_unique_idx")]
  pub alg: String,
  pub r#use: Option<String>,
  pub key_ops: Option<String>,
  pub n: Option<String>,
  pub e: Option<String>,
  pub d: Option<String>,
  pub p: Option<String>,
  pub q: Option<String>,
  pub dp: Option<String>,
  pub dq: Option<String>,
  pub qi: Option<String>,
  pub crv: Option<String>,
  pub x: Option<String>,
  pub y: Option<String>,
  pub d_ec: Option<String>,
  pub k: Option<String>,
  pub x5u: Option<String>,
  pub x5c: Option<String>,
  pub x5t: Option<String>,
  pub x5t_s256: Option<String>,
  pub updated_at: i64,
  pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
  pub fn key_operations(&self) -> Vec<String> {
    if let Some(key_ops) = self.key_ops.as_ref() {
      match serde_json::from_str::<Vec<String>>(key_ops.as_str()) {
        Ok(key_ops_array) => return key_ops_array,
        Err(e) => {
          log::error!("invalid key_ops JSON: {}", e);
        }
      }
    }
    Vec::new()
  }

  pub fn public_key_operations(&self) -> Option<Vec<String>> {
    let key_operations: Vec<String> = self
      .key_operations()
      .into_iter()
      .filter(|op| matches!(op.as_str(), "verify" | "encrypt" | "wrapKey"))
      .collect();

    if key_operations.is_empty() {
      return None;
    }
    Some(key_operations)
  }
}
