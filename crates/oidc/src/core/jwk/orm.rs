use os_model::entities::{prelude::*, *};
use sea_orm::*;

pub fn model_to_jwt_jwk(model: jwks::Model) -> Result<jsonwebtoken::jwk::Jwk, serde_json::Error> {
  use jsonwebtoken::jwk::*;

  let key_algorithm = if !model.alg.is_empty() {
    Some(match model.alg.as_str() {
      "HS256" => KeyAlgorithm::HS256,
      "HS384" => KeyAlgorithm::HS384,
      "HS512" => KeyAlgorithm::HS512,
      "RS256" => KeyAlgorithm::RS256,
      "RS384" => KeyAlgorithm::RS384,
      "RS512" => KeyAlgorithm::RS512,
      "PS256" => KeyAlgorithm::PS256,
      "PS384" => KeyAlgorithm::PS384,
      "PS512" => KeyAlgorithm::PS512,
      "ES256" => KeyAlgorithm::ES256,
      "ES384" => KeyAlgorithm::ES384,
      "EdDSA" => KeyAlgorithm::EdDSA,
      _ => KeyAlgorithm::RS256,
    })
  } else {
    None
  };

  let public_key_use = model.r#use.map(|u| match u.as_str() {
    "sig" => PublicKeyUse::Signature,
    "enc" => PublicKeyUse::Encryption,
    other => PublicKeyUse::Other(other.to_string()),
  });

  let key_operations = model
    .key_ops
    .and_then(|ko| serde_json::from_str::<Vec<String>>(&ko).ok())
    .and_then(|ops| {
      let parsed: Vec<KeyOperations> = ops
        .iter()
        .filter_map(|op| match op.as_str() {
          "sign" => Some(KeyOperations::Sign),
          "verify" => Some(KeyOperations::Verify),
          "encrypt" => Some(KeyOperations::Encrypt),
          "decrypt" => Some(KeyOperations::Decrypt),
          "wrapKey" => Some(KeyOperations::WrapKey),
          "unwrapKey" => Some(KeyOperations::UnwrapKey),
          "deriveKey" => Some(KeyOperations::DeriveKey),
          "deriveBits" => Some(KeyOperations::DeriveBits),
          _ => None,
        })
        .collect();
      if parsed.is_empty() {
        None
      } else {
        Some(parsed)
      }
    });

  let x509_chain = model
    .x5c
    .and_then(|chain| serde_json::from_str::<Vec<String>>(&chain).ok());

  let common = CommonParameters {
    public_key_use,
    key_operations,
    key_algorithm,
    key_id: Some(model.kid.to_string()),
    x509_url: model.x5u,
    x509_chain,
    x509_sha1_fingerprint: model.x5t,
    x509_sha256_fingerprint: model.x5t_s256,
  };

  let algorithm = match model.kty.as_str() {
    "EC" => {
      let curve = model
        .crv
        .and_then(|c| match c.as_str() {
          "P256" | "P-256" => Some(EllipticCurve::P256),
          "P384" | "P-384" => Some(EllipticCurve::P384),
          "P521" | "P-521" => Some(EllipticCurve::P521),
          "Ed25519" => Some(EllipticCurve::Ed25519),
          _ => None,
        })
        .unwrap_or(EllipticCurve::P256);

      AlgorithmParameters::EllipticCurve(EllipticCurveKeyParameters {
        key_type: EllipticCurveKeyType::EC,
        curve,
        x: model.x.unwrap_or_default(),
        y: model.y.unwrap_or_default(),
      })
    }
    "RSA" => AlgorithmParameters::RSA(RSAKeyParameters {
      key_type: RSAKeyType::RSA,
      n: model.n.unwrap_or_default(),
      e: model.e.unwrap_or_default(),
    }),
    "oct" => AlgorithmParameters::OctetKey(OctetKeyParameters {
      key_type: OctetKeyType::Octet,
      value: model.k.unwrap_or_default(),
    }),
    "OKP" => {
      let curve = model
        .crv
        .and_then(|c| match c.as_str() {
          "Ed25519" => Some(EllipticCurve::Ed25519),
          "P256" | "P-256" => Some(EllipticCurve::P256),
          "P384" | "P-384" => Some(EllipticCurve::P384),
          "P521" | "P-521" => Some(EllipticCurve::P521),
          _ => None,
        })
        .unwrap_or(EllipticCurve::Ed25519);

      AlgorithmParameters::OctetKeyPair(OctetKeyPairParameters {
        key_type: OctetKeyPairType::OctetKeyPair,
        curve,
        x: model.x.unwrap_or_default(),
      })
    }
    _ => AlgorithmParameters::RSA(RSAKeyParameters {
      key_type: RSAKeyType::RSA,
      n: model.n.unwrap_or_default(),
      e: model.e.unwrap_or_default(),
    }),
  };

  Ok(Jwk { common, algorithm })
}

pub fn jwk_to_model(jwk: jsonwebtoken::jwk::Jwk) -> jwks::Model {
  let now = chrono::Utc::now().timestamp();
  let common_alg = jwk
    .common
    .key_algorithm
    .map(|ka| format!("{:?}", ka))
    .unwrap_or_default();
  let common_use = jwk.common.public_key_use.map(|pku| match pku {
    jsonwebtoken::jwk::PublicKeyUse::Signature => "sig".to_string(),
    jsonwebtoken::jwk::PublicKeyUse::Encryption => "enc".to_string(),
    jsonwebtoken::jwk::PublicKeyUse::Other(other) => other,
  });
  let common_key_ops = jwk
    .common
    .key_operations
    .as_ref()
    .map(|ko| serde_json::to_string(ko).unwrap_or_else(|_| "[]".to_string()));
  let common_x5c = jwk
    .common
    .x509_chain
    .map(|chain| serde_json::to_string(&chain).unwrap_or_default());

  let mut model = jwks::Model {
    active: 1,
    alg: common_alg,
    r#use: common_use,
    key_ops: common_key_ops,
    x5u: jwk.common.x509_url,
    x5c: common_x5c,
    x5t: jwk.common.x509_sha1_fingerprint,
    x5t_s256: jwk.common.x509_sha256_fingerprint,
    updated_at: now,
    created_at: now,
    ..Default::default()
  };

  match jwk.algorithm {
    jsonwebtoken::jwk::AlgorithmParameters::EllipticCurve(ref ec_params) => {
      model.kty = "EC".to_string();
      model.crv = Some(format!("{:?}", ec_params.curve));
      model.x = Some(ec_params.x.clone());
      model.y = Some(ec_params.y.clone());
    }
    jsonwebtoken::jwk::AlgorithmParameters::RSA(ref rsa_params) => {
      model.kty = "RSA".to_string();
      model.n = Some(rsa_params.n.clone());
      model.e = Some(rsa_params.e.clone());
    }
    jsonwebtoken::jwk::AlgorithmParameters::OctetKey(ref octet_key_parameters) => {
      model.kty = "oct".to_string();
      model.k = Some(octet_key_parameters.value.clone());
    }
    jsonwebtoken::jwk::AlgorithmParameters::OctetKeyPair(ref octet_key_pair_parameters) => {
      model.kty = "OKP".to_string();
      model.crv = Some(format!("{:?}", octet_key_pair_parameters.curve));
      model.x = Some(octet_key_pair_parameters.x.clone());
    }
  }

  model
}

pub async fn list_jwks(db: &DatabaseConnection) -> Result<Vec<jwks::Model>, DbErr> {
  Jwks::find()
    .filter(jwks::Column::Active.ne(0))
    .all(db)
    .await
}

pub async fn get_jwk_by_kid(
  db: &DatabaseConnection,
  kid: impl Into<String>,
) -> Result<Option<jwks::Model>, DbErr> {
  let kid_str = kid.into();
  let kid_i64: i64 = kid_str
    .parse()
    .map_err(|_| DbErr::Custom("Invalid kid format".to_string()))?;

  Jwks::find_by_id(kid_i64)
    .filter(jwks::Column::Active.ne(0))
    .one(db)
    .await
}

pub async fn get_jwk_for_sign_and_verify(
  db: &DatabaseConnection,
) -> Result<Option<jwks::Model>, DbErr> {
  let jwks = list_jwks(db).await?;
  Ok(jwks.into_iter().find(|jwk| {
    if let Some(key_ops) = jwk.key_ops.as_ref() {
      key_ops.contains("\"sign\"") && key_ops.contains("\"verify\"")
    } else {
      false
    }
  }))
}

pub async fn create_jwk(db: &DatabaseConnection, jwk: jwks::Model) -> Result<jwks::Model, DbErr> {
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

  new_jwk.insert(db).await
}

// Convenience function for converting jwks::Model to JWT Jwk
pub fn jwk_model_to_jwt_jwk(row: jwks::Model) -> Result<jsonwebtoken::jwk::Jwk, serde_json::Error> {
  model_to_jwt_jwk(row)
}
