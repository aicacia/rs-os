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

#[derive(sqlx::FromRow, Clone, Default, Serialize, Deserialize)]
pub struct JwkSQLRow {
  #[sqlx(default)]
  #[serde(default)]
  #[serde(serialize_with = "serialize_i64_as_string")]
  #[serde(deserialize_with = "deserialize_string_as_i64")]
  pub kid: i64,
  #[sqlx(default)]
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

  #[sqlx(default)]
  #[serde(default)]
  pub updated_at: i64,
  #[sqlx(default)]
  #[serde(default)]
  pub created_at: i64,
}

impl JwkSQLRow {
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

impl TryInto<jsonwebtoken::jwk::Jwk> for JwkSQLRow {
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

impl TryFrom<jsonwebtoken::jwk::Jwk> for JwkSQLRow {
  type Error = serde_json::Error;

  fn try_from(value: jsonwebtoken::jwk::Jwk) -> Result<Self, Self::Error> {
    Ok(serde_json::from_value(serde_json::to_value(value)?)?)
  }
}

pub async fn list_jwks(pool: &sqlx::AnyPool) -> sqlx::Result<Vec<JwkSQLRow>> {
  sqlx::query_as(r#"SELECT * FROM jwks WHERE "active" != 0;"#)
    .fetch_all(pool)
    .await
}

pub async fn get_jwk_by_kid(
  pool: &sqlx::AnyPool,
  kid: impl Into<String>,
) -> sqlx::Result<Option<JwkSQLRow>> {
  sqlx::query_as(r#"SELECT * FROM jwks WHERE "kid" = $1 AND "active" != 0;"#)
    .bind(kid.into())
    .fetch_optional(pool)
    .await
}

pub async fn get_jwk_for_sign_and_verify(pool: &sqlx::AnyPool) -> sqlx::Result<Option<JwkSQLRow>> {
  Ok(list_jwks(pool).await?.into_iter().find(|jwk| {
    if let Some(key_ops) = jwk.key_ops.as_ref() {
      key_ops.contains("\"sign\"") && key_ops.contains("\"verify\"")
    } else {
      false
    }
  }))
}

pub async fn create_jwk(pool: &sqlx::AnyPool, jwk: JwkSQLRow) -> sqlx::Result<JwkSQLRow> {
  sqlx::query_as::<_, JwkSQLRow>(
    r#"INSERT INTO jwks (
            kty, alg, use, key_ops,
            n, e, d, p, q, dp, dq, qi,
            crv, x, y, d_ec,
            k,
            x5u, x5c, x5t, x5t_s256
        )
        VALUES ($1, $2, $3, $4,
                $5, $6, $7, $8, $9, $10, $11, $12,
                $13, $14, $15, $16,
                $17,
                $18, $19, $20, $21)
        RETURNING *;"#,
  )
  .bind(&jwk.kty)
  .bind(&jwk.alg)
  .bind(&jwk.r#use)
  .bind(&jwk.key_ops)
  .bind(&jwk.n)
  .bind(&jwk.e)
  .bind(&jwk.d)
  .bind(&jwk.p)
  .bind(&jwk.q)
  .bind(&jwk.dp)
  .bind(&jwk.dq)
  .bind(&jwk.qi)
  .bind(&jwk.crv)
  .bind(&jwk.x)
  .bind(&jwk.y)
  .bind(&jwk.d_ec)
  .bind(&jwk.k)
  .bind(&jwk.x5u)
  .bind(&jwk.x5c)
  .bind(&jwk.x5t)
  .bind(&jwk.x5t_s256)
  .fetch_one(pool)
  .await
}
