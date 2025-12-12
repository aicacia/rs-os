use std::{io, str::FromStr};

use base64::{DecodeError, Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use ed25519_dalek::{SigningKey, pkcs8::EncodePrivateKey};
use jsonwebtoken::{Algorithm, EncodingKey};
use num_bigint_dig::traits::ModInverse;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use p256::pkcs8::EncodePrivateKey as P256EncodePrivateKey;
use p384::pkcs8::EncodePrivateKey as P384EncodePrivateKey;
use rand::RngCore;
use rsa::{
  RsaPrivateKey, RsaPublicKey,
  pkcs1::EncodeRsaPrivateKey as RSAEncodePrivateKey,
  traits::{PrivateKeyParts, PublicKeyParts},
};
use sea_orm::Set;
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

pub fn model_to_jwt_jwk(model: Model) -> Result<jsonwebtoken::jwk::Jwk, serde_json::Error> {
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

pub fn jwk_to_model(jwk: jsonwebtoken::jwk::Jwk) -> Model {
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

  let mut model = Model {
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

pub async fn list_jwks(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
  Entity::find().filter(Column::Active.ne(0)).all(db).await
}

pub async fn get_jwk_by_kid(
  db: &DatabaseConnection,
  kid: impl Into<String>,
) -> Result<Option<Model>, DbErr> {
  let kid_str = kid.into();
  let kid_i64: i64 = kid_str
    .parse()
    .map_err(|_| DbErr::Custom("Invalid kid format".to_string()))?;

  Entity::find_by_id(kid_i64)
    .filter(Column::Active.ne(0))
    .one(db)
    .await
}

pub async fn get_jwk_for_sign_and_verify(db: &DatabaseConnection) -> Result<Option<Model>, DbErr> {
  let jwks = list_jwks(db).await?;
  Ok(jwks.into_iter().find(|jwk| {
    if let Some(key_ops) = jwk.key_ops.as_ref() {
      key_ops.contains("\"sign\"") && key_ops.contains("\"verify\"")
    } else {
      false
    }
  }))
}

pub async fn create_jwk(db: &DatabaseConnection, jwk: Model) -> Result<Model, DbErr> {
  let now = chrono::Utc::now().timestamp();
  let new_jwk = ActiveModel {
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

fn generate_hmac_jwk(alg: Algorithm) -> Result<Model, io::Error> {
  let (alg_name, key_len) = match alg {
    Algorithm::HS256 => ("HS256".to_owned(), 32), // 256-bit
    Algorithm::HS384 => ("HS384".to_owned(), 48), // 384-bit
    Algorithm::HS512 => ("HS512".to_owned(), 64), // 512-bit
    _ => return Err(io::Error::other(format!("unsupported HMAC: {:?}", alg))),
  };

  let mut secret: Vec<u8> = vec![0u8; key_len];
  rand::thread_rng().fill_bytes(&mut secret);

  let k = BASE64_URL_SAFE_NO_PAD.encode(&secret);

  let jwk = Model {
    active: 1,
    alg: alg_name,
    kty: "oct".to_owned(),
    r#use: Some("sig".to_owned()),
    key_ops: Some("[\"sign\",\"verify\"]".to_owned()),
    k: Some(k),
    ..Default::default()
  };

  Ok(jwk)
}

fn generate_es_jwk(alg: Algorithm) -> Result<Model, io::Error> {
  match alg {
    Algorithm::ES256 => {
      let secret_key = p256::elliptic_curve::SecretKey::random(&mut rand::thread_rng());
      let d = BASE64_URL_SAFE_NO_PAD.encode(secret_key.to_bytes());

      let public_key: p256::elliptic_curve::PublicKey<p256::NistP256> = secret_key.public_key();
      let affine = public_key.as_affine();
      let point = affine.to_encoded_point(false);

      let x = BASE64_URL_SAFE_NO_PAD.encode(
        &point
          .x()
          .ok_or_else(|| io::Error::other("invalid x"))?
          .to_vec(),
      );
      let y = BASE64_URL_SAFE_NO_PAD.encode(
        &point
          .y()
          .ok_or_else(|| io::Error::other("invalid y"))?
          .to_vec(),
      );

      let jwk = Model {
        active: 1,
        alg: "ES256".to_owned(),
        kty: "EC".to_owned(),
        r#use: Some("sig".to_owned()),
        key_ops: Some("[\"sign\",\"verify\"]".to_owned()),
        crv: Some("P-256".to_owned()),
        d: Some(d),
        x: Some(x),
        y: Some(y),
        ..Default::default()
      };

      Ok(jwk)
    }
    Algorithm::ES384 => {
      use p384::elliptic_curve::sec1::ToEncodedPoint;

      let secret_key = p384::elliptic_curve::SecretKey::random(&mut rand::thread_rng());
      let d = BASE64_URL_SAFE_NO_PAD.encode(secret_key.to_bytes());

      let public_key: p384::elliptic_curve::PublicKey<p384::NistP384> = secret_key.public_key();
      let affine = public_key.as_affine();
      let point = affine.to_encoded_point(false);

      let x = BASE64_URL_SAFE_NO_PAD.encode(
        &point
          .x()
          .ok_or_else(|| io::Error::other("invalid x"))?
          .to_vec(),
      );
      let y = BASE64_URL_SAFE_NO_PAD.encode(
        &point
          .y()
          .ok_or_else(|| io::Error::other("invalid y"))?
          .to_vec(),
      );

      let jwk = Model {
        active: 1,
        alg: "ES384".to_owned(),
        kty: "EC".to_owned(),
        r#use: Some("sig".to_owned()),
        key_ops: Some("[\"sign\",\"verify\"]".to_owned()),
        crv: Some("P-384".to_owned()),
        d: Some(d),
        x: Some(x),
        y: Some(y),
        ..Default::default()
      };

      Ok(jwk)
    }
    _ => Err(io::Error::other(format!("unsupported EC: {:?}", alg))),
  }
}

fn generate_rsa_jwk(alg: Algorithm) -> Result<Model, io::Error> {
  let (alg_name, key_size) = match alg {
    Algorithm::RS256 => ("RS256".to_owned(), 2048),
    Algorithm::RS384 => ("RS384".to_owned(), 3072),
    Algorithm::RS512 => ("RS512".to_owned(), 4096),
    _ => return Err(io::Error::other(format!("unsupported RSA: {:?}", alg))),
  };

  let mut rng = rand::thread_rng();

  let private_key = RsaPrivateKey::new(&mut rng, key_size).map_err(io::Error::other)?;
  let public_key = RsaPublicKey::from(&private_key);

  let primes = private_key.primes();
  let p = &primes[0];
  let q = &primes[1];

  let dp = private_key.d() % (p - 1u8);
  let dq = private_key.d() % (q - 1u8);
  let qi = q
    .mod_inverse(p)
    .ok_or_else(|| io::Error::other("inverse should exist"))?;
  let (_, qi_bytes) = qi.to_bytes_be();

  let n = BASE64_URL_SAFE_NO_PAD.encode(&public_key.n().to_bytes_be());
  let e = BASE64_URL_SAFE_NO_PAD.encode(&public_key.e().to_bytes_be());
  let d = BASE64_URL_SAFE_NO_PAD.encode(&private_key.d().to_bytes_be());
  let p = BASE64_URL_SAFE_NO_PAD.encode(&p.to_bytes_be());
  let q = BASE64_URL_SAFE_NO_PAD.encode(&q.to_bytes_be());
  let dp = BASE64_URL_SAFE_NO_PAD.encode(&dp.to_bytes_be());
  let dq = BASE64_URL_SAFE_NO_PAD.encode(&dq.to_bytes_be());
  let qi = BASE64_URL_SAFE_NO_PAD.encode(&qi_bytes);

  let jwk_model = Model {
    active: 1,
    alg: alg_name,
    kty: "RSA".to_owned(),
    r#use: Some("sig".to_owned()),
    key_ops: Some("[\"sign\",\"verify\"]".to_owned()),
    n: Some(n),
    e: Some(e),
    d: Some(d),
    p: Some(p),
    q: Some(q),
    dp: Some(dp),
    dq: Some(dq),
    qi: Some(qi),
    ..Default::default()
  };

  Ok(jwk_model)
}

fn generate_ed_jwk() -> Result<Model, io::Error> {
  use ed25519_dalek::SigningKey;

  let signing_key = SigningKey::generate(&mut rand::thread_rng());
  let verifying_key = signing_key.verifying_key();

  // Encode private (d) and public (x) parts
  let d = BASE64_URL_SAFE_NO_PAD.encode(signing_key.to_bytes());
  let x = BASE64_URL_SAFE_NO_PAD.encode(verifying_key.to_bytes());

  // Construct JWK row
  let jwk = Model {
    active: 1,
    alg: "EdDSA".to_owned(),
    kty: "OKP".to_owned(), // Octet Key Pair
    r#use: Some("sig".to_owned()),
    key_ops: Some("[\"sign\",\"verify\"]".to_owned()),
    crv: Some("Ed25519".to_owned()),
    d: Some(d),
    x: Some(x),
    ..Default::default()
  };

  Ok(jwk)
}

pub fn generate_jwk(alg: Algorithm) -> Result<Model, io::Error> {
  match alg {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => generate_hmac_jwk(alg),
    Algorithm::ES256 | Algorithm::ES384 => generate_es_jwk(alg),
    Algorithm::RS256
    | Algorithm::RS384
    | Algorithm::RS512
    | Algorithm::PS256
    | Algorithm::PS384
    | Algorithm::PS512 => generate_rsa_jwk(alg),
    Algorithm::EdDSA => generate_ed_jwk(),
  }
}

fn encoding_key_hmac(jwk_model: &Model) -> Result<EncodingKey, DecodeError> {
  let secret = BASE64_URL_SAFE_NO_PAD.decode(jwk_model.k.clone().unwrap_or_default())?;
  Ok(EncodingKey::from_secret(secret.as_ref()))
}

fn encoding_key_es256(jwk_model: &Model) -> Result<EncodingKey, io::Error> {
  let d = jwk_model
    .d
    .as_ref()
    .ok_or_else(|| io::Error::other("missing private key component 'd' for ES256"))?;
  let d_bytes = BASE64_URL_SAFE_NO_PAD.decode(d).map_err(io::Error::other)?;
  let secret_key: p256::elliptic_curve::SecretKey<p256::NistP256> =
    p256::elliptic_curve::SecretKey::from_slice(&d_bytes)
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
  Ok(EncodingKey::from_ec_der(
    P256EncodePrivateKey::to_pkcs8_der(&secret_key)
      .map_err(io::Error::other)?
      .as_bytes(),
  ))
}

fn encoding_key_es384(jwk_model: &Model) -> Result<EncodingKey, io::Error> {
  let d = jwk_model
    .d
    .as_ref()
    .ok_or_else(|| io::Error::other("missing private key component 'd' for ES384"))?;
  let d_bytes = BASE64_URL_SAFE_NO_PAD.decode(d).map_err(io::Error::other)?;
  let secret_key: p384::elliptic_curve::SecretKey<p384::NistP384> =
    p384::elliptic_curve::SecretKey::from_slice(&d_bytes)
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
  Ok(EncodingKey::from_ec_der(
    P384EncodePrivateKey::to_pkcs8_der(&secret_key)
      .map_err(io::Error::other)?
      .as_bytes(),
  ))
}

fn encoding_key_rsa(jwk_model: &Model) -> Result<EncodingKey, io::Error> {
  use num_bigint_dig::BigUint;

  let n_b64 = jwk_model
    .n
    .as_ref()
    .ok_or_else(|| io::Error::other("missing 'n' for RSA"))?;
  let e_b64 = jwk_model
    .e
    .as_ref()
    .ok_or_else(|| io::Error::other("missing 'e' for RSA"))?;
  let d_b64 = jwk_model
    .d
    .as_ref()
    .ok_or_else(|| io::Error::other("missing 'd' for RSA"))?;
  let p_b64 = jwk_model
    .p
    .as_ref()
    .ok_or_else(|| io::Error::other("missing 'p' for RSA"))?;
  let q_b64 = jwk_model
    .q
    .as_ref()
    .ok_or_else(|| io::Error::other("missing 'q' for RSA"))?;
  let n = BigUint::from_bytes_be(
    &BASE64_URL_SAFE_NO_PAD
      .decode(n_b64)
      .map_err(io::Error::other)?,
  );
  let e = BigUint::from_bytes_be(
    &BASE64_URL_SAFE_NO_PAD
      .decode(e_b64)
      .map_err(io::Error::other)?,
  );
  let d = BigUint::from_bytes_be(
    &BASE64_URL_SAFE_NO_PAD
      .decode(d_b64)
      .map_err(io::Error::other)?,
  );
  let p = BigUint::from_bytes_be(
    &BASE64_URL_SAFE_NO_PAD
      .decode(p_b64)
      .map_err(io::Error::other)?,
  );
  let q = BigUint::from_bytes_be(
    &BASE64_URL_SAFE_NO_PAD
      .decode(q_b64)
      .map_err(io::Error::other)?,
  );
  let rsa_key = RsaPrivateKey::from_components(n, e, d, vec![p, q]).map_err(io::Error::other)?;

  Ok(EncodingKey::from_rsa_der(
    RSAEncodePrivateKey::to_pkcs1_der(&rsa_key)
      .map_err(io::Error::other)?
      .as_bytes(),
  ))
}

fn encoding_key_ed(jwk_model: &Model) -> Result<EncodingKey, io::Error> {
  let d = jwk_model
    .d
    .as_ref()
    .ok_or_else(|| io::Error::other("missing private key component 'd' for EdDSA"))?;
  let d_bytes = BASE64_URL_SAFE_NO_PAD.decode(d).map_err(io::Error::other)?;

  let signing_key = SigningKey::from_bytes(
    &d_bytes
      .try_into()
      .map_err(|e| format!("invalid bytes: {:?}", e))
      .map_err(io::Error::other)?,
  );

  let der = signing_key.to_pkcs8_der().map_err(io::Error::other)?;

  Ok(EncodingKey::from_ed_der(der.as_bytes()))
}

pub fn to_encoding_key(jwk_model: &Model) -> Result<EncodingKey, io::Error> {
  let alg = Algorithm::from_str(jwk_model.alg.as_str()).map_err(io::Error::other)?;

  match alg {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
      encoding_key_hmac(jwk_model).map_err(io::Error::other)
    }
    Algorithm::ES256 => encoding_key_es256(jwk_model),
    Algorithm::ES384 => encoding_key_es384(jwk_model),
    Algorithm::RS256
    | Algorithm::RS384
    | Algorithm::RS512
    | Algorithm::PS256
    | Algorithm::PS384
    | Algorithm::PS512 => encoding_key_rsa(jwk_model),
    Algorithm::EdDSA => encoding_key_ed(jwk_model),
  }
}

pub fn is_public_key_op(key_op: &String) -> bool {
  match key_op.as_str() {
    "verify" | "encrypt" | "wrapKey" => true,
    _ => false,
  }
}
