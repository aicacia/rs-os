use os_model::entities::{prelude::*, *};
use sea_orm::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

use crate::core::jwk::helper::is_public_key_op;

fn serialize_i64_as_string<S>(value: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&value.to_string())
}

fn deserialize_string_as_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
  D: Deserializer<'de>,
{
  String::deserialize(deserializer)?
    .parse()
    .map_err(serde::de::Error::custom)
}

fn deserialize_optional_vec_of_strings_as_string<'de, D>(
  deserializer: D,
) -> Result<Option<String>, D::Error>
where
  D: Deserializer<'de>,
{
  if let Some(vec_of_strings) = Option::<Vec<String>>::deserialize(deserializer)? {
    let string = serde_json::to_string(&vec_of_strings).map_err(serde::de::Error::custom)?;
    Ok(Some(string))
  } else {
    Ok(None)
  }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct JwkRow {
  #[serde(default)]
  #[serde(serialize_with = "serialize_i64_as_string")]
  #[serde(deserialize_with = "deserialize_string_as_i64")]
  pub kid: i64,
  #[serde(default)]
  pub active: i64,

  pub kty: String,
  pub alg: String,
  pub r#use: Option<String>,
  #[serde(deserialize_with = "deserialize_optional_vec_of_strings_as_string")]
  pub key_ops: Option<String>,

  // RSA fields
  pub n: Option<String>,
  pub e: Option<String>,
  pub d: Option<String>,
  pub p: Option<String>,
  pub q: Option<String>,
  pub dp: Option<String>,
  pub dq: Option<String>,
  pub qi: Option<String>,

  // EC fields
  pub crv: Option<String>,
  pub x: Option<String>,
  pub y: Option<String>,
  pub d_ec: Option<String>,

  // Symmetric (oct) fields
  pub k: Option<String>,

  // X.509 fields
  pub x5u: Option<String>,
  pub x5c: Option<String>,
  pub x5t: Option<String>,
  pub x5t_s256: Option<String>,

  #[serde(default)]
  pub updated_at: i64,
  #[serde(default)]
  pub created_at: i64,
}

impl From<jwks::Model> for JwkRow {
  fn from(model: jwks::Model) -> Self {
    Self {
      kid: model.kid,
      active: model.active,
      kty: model.kty,
      alg: model.alg,
      r#use: model.r#use,
      key_ops: model.key_ops,
      n: model.n,
      e: model.e,
      d: model.d,
      p: model.p,
      q: model.q,
      dp: model.dp,
      dq: model.dq,
      qi: model.qi,
      crv: model.crv,
      x: model.x,
      y: model.y,
      d_ec: model.d_ec,
      k: model.k,
      x5u: model.x5u,
      x5c: model.x5c,
      x5t: model.x5t,
      x5t_s256: model.x5t_s256,
      updated_at: model.updated_at,
      created_at: model.created_at,
    }
  }
}

impl Into<jwks::Model> for JwkRow {
  fn into(self) -> jwks::Model {
    jwks::Model {
      kid: self.kid,
      active: self.active,
      kty: self.kty,
      alg: self.alg,
      r#use: self.r#use,
      key_ops: self.key_ops,
      n: self.n,
      e: self.e,
      d: self.d,
      p: self.p,
      q: self.q,
      dp: self.dp,
      dq: self.dq,
      qi: self.qi,
      crv: self.crv,
      x: self.x,
      y: self.y,
      d_ec: self.d_ec,
      k: self.k,
      x5u: self.x5u,
      x5c: self.x5c,
      x5t: self.x5t,
      x5t_s256: self.x5t_s256,
      updated_at: self.updated_at,
      created_at: self.created_at,
    }
  }
}

impl JwkRow {
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
      .filter(is_public_key_op)
      .collect();

    if key_operations.is_empty() {
      return None;
    }
    Some(key_operations)
  }
}

impl TryInto<jsonwebtoken::jwk::Jwk> for JwkRow {
  type Error = serde_json::Error;

  fn try_into(self) -> Result<jsonwebtoken::jwk::Jwk, Self::Error> {
    let value = match serde_json::to_value(&self)? {
      serde_json::Value::Object(mut map) => {
        if let Some(serde_json::Value::String(key_ops)) = map.remove("key_ops") {
          map.insert(
            "key_ops".to_owned(),
            serde_json::from_str(key_ops.as_str())?,
          );
        }
        serde_json::Value::Object(map)
      }
      _ => {
        return Err(serde_json::Error::custom(
          "expected a JSON object but got a different type",
        ));
      }
    };
    Ok(serde_json::from_value(value)?)
  }
}

impl TryFrom<jsonwebtoken::jwk::Jwk> for JwkRow {
  type Error = serde_json::Error;

  fn try_from(value: jsonwebtoken::jwk::Jwk) -> Result<Self, Self::Error> {
    Ok(serde_json::from_value(serde_json::to_value(value)?)?)
  }
}

pub async fn list_jwks(db: &DatabaseConnection) -> Result<Vec<JwkRow>, DbErr> {
  let models = Jwks::find()
    .filter(jwks::Column::Active.ne(0))
    .all(db)
    .await?;

  Ok(models.into_iter().map(|m| m.into()).collect())
}

pub async fn get_jwk_by_kid(
  db: &DatabaseConnection,
  kid: impl Into<String>,
) -> Result<Option<JwkRow>, DbErr> {
  let kid_str = kid.into();
  let kid_i64: i64 = kid_str
    .parse()
    .map_err(|_| DbErr::Custom("Invalid kid format".to_string()))?;

  let model = Jwks::find_by_id(kid_i64)
    .filter(jwks::Column::Active.ne(0))
    .one(db)
    .await?;

  Ok(model.map(|m| m.into()))
}

pub async fn get_jwk_for_sign_and_verify(db: &DatabaseConnection) -> Result<Option<JwkRow>, DbErr> {
  let jwks = list_jwks(db).await?;
  Ok(jwks.into_iter().find(|jwk| {
    if let Some(key_ops) = jwk.key_ops.as_ref() {
      key_ops.contains("\"sign\"") && key_ops.contains("\"verify\"")
    } else {
      false
    }
  }))
}

pub async fn create_jwk(db: &DatabaseConnection, jwk: JwkRow) -> Result<JwkRow, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let new_jwk = jwks::ActiveModel {
    kty: Set(jwk.kty),
    alg: Set(jwk.alg),
    r#use: Set(jwk.r#use),
    key_ops: Set(jwk.key_ops),
    n: Set(jwk.n),
    e: Set(jwk.e),
    d: Set(jwk.d),
    p: Set(jwk.p),
    q: Set(jwk.q),
    dp: Set(jwk.dp),
    dq: Set(jwk.dq),
    qi: Set(jwk.qi),
    crv: Set(jwk.crv),
    x: Set(jwk.x),
    y: Set(jwk.y),
    d_ec: Set(jwk.d_ec),
    k: Set(jwk.k),
    x5u: Set(jwk.x5u),
    x5c: Set(jwk.x5c),
    x5t: Set(jwk.x5t),
    x5t_s256: Set(jwk.x5t_s256),
    active: Set(1),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };

  let model = new_jwk.insert(db).await?;
  Ok(model.into())
}
