use std::{error::Error, io, str::FromStr};

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, EncodingKey};
use num_bigint_dig::traits::ModInverse;
use p256::pkcs8::EncodePrivateKey as P256EncodePrivateKey;
use p384::pkcs8::EncodePrivateKey as P384EncodePrivateKey;
use rand::RngCore;
use rsa::{
  RsaPrivateKey, RsaPublicKey,
  pkcs1::EncodeRsaPrivateKey as RSAEncodePrivateKey,
  traits::{PrivateKeyParts, PublicKeyParts},
};

use crate::core::jwk::sql::{JwkSQLRow, create_jwk, list_jwks};

pub async fn init_jwk(pool: &sqlx::AnyPool) -> Result<(), Box<dyn Error>> {
  let jwks = list_jwks(pool).await?;

  if jwks.is_empty() {
    let _ = create_jwk(pool, generate_jwk(Algorithm::EdDSA)?).await?;
  }

  Ok(())
}

fn generate_hmac_jwk(alg: Algorithm) -> Result<JwkSQLRow, Box<dyn std::error::Error>> {
  let (alg_name, key_len) = match alg {
    Algorithm::HS256 => ("HS256".to_owned(), 32), // 256-bit
    Algorithm::HS384 => ("HS384".to_owned(), 48), // 384-bit
    Algorithm::HS512 => ("HS512".to_owned(), 64), // 512-bit
    _ => return Err(format!("unsupported HMAC: {:?}", alg).into()),
  };

  let mut secret: Vec<u8> = vec![0u8; key_len];
  rand::thread_rng().fill_bytes(&mut secret);

  let k = BASE64_URL_SAFE_NO_PAD.encode(&secret);

  let mut jwk = JwkSQLRow::default();
  jwk.alg = alg_name;
  jwk.kty = "oct".to_owned();
  jwk.r#use = Some("sig".to_owned());
  jwk.key_ops = Some("[\"sign\",\"verify\"]".to_owned());
  jwk.k = Some(k);

  Ok(jwk)
}

fn generate_es_jwk(alg: Algorithm) -> Result<JwkSQLRow, Box<dyn std::error::Error>> {
  match alg {
    Algorithm::ES256 => {
      use p256::elliptic_curve::sec1::ToEncodedPoint;

      let secret_key = p256::elliptic_curve::SecretKey::random(&mut rand::thread_rng());
      let d = BASE64_URL_SAFE_NO_PAD.encode(secret_key.to_bytes());

      let public_key: p256::elliptic_curve::PublicKey<p256::NistP256> = secret_key.public_key();
      let affine = public_key.as_affine();
      let point = affine.to_encoded_point(false);

      let x = BASE64_URL_SAFE_NO_PAD.encode(&point.x().ok_or_else(|| "invalid x")?.to_vec());
      let y = BASE64_URL_SAFE_NO_PAD.encode(&point.y().ok_or_else(|| "invalid y")?.to_vec());

      let mut jwk = JwkSQLRow::default();
      jwk.alg = "ES256".to_owned();
      jwk.kty = "EC".to_owned();
      jwk.r#use = Some("sig".to_owned());
      jwk.key_ops = Some("[\"sign\",\"verify\"]".to_owned());
      jwk.crv = Some("P-256".to_owned());
      jwk.d = Some(d);
      jwk.x = Some(x);
      jwk.y = Some(y);

      Ok(jwk)
    }
    Algorithm::ES384 => {
      use p384::elliptic_curve::sec1::ToEncodedPoint;

      let secret_key = p384::elliptic_curve::SecretKey::random(&mut rand::thread_rng());
      let d = BASE64_URL_SAFE_NO_PAD.encode(secret_key.to_bytes());

      let public_key: p384::elliptic_curve::PublicKey<p384::NistP384> = secret_key.public_key();
      let affine = public_key.as_affine();
      let point = affine.to_encoded_point(false);

      let x = BASE64_URL_SAFE_NO_PAD.encode(&point.x().ok_or_else(|| "invalid x")?.to_vec());
      let y = BASE64_URL_SAFE_NO_PAD.encode(&point.y().ok_or_else(|| "invalid y")?.to_vec());

      let mut jwk = JwkSQLRow::default();
      jwk.alg = "ES384".to_owned();
      jwk.kty = "EC".to_owned();
      jwk.r#use = Some("sig".to_owned());
      jwk.key_ops = Some("[\"sign\",\"verify\"]".to_owned());
      jwk.crv = Some("P-384".to_owned());
      jwk.d = Some(d);
      jwk.x = Some(x);
      jwk.y = Some(y);

      Ok(jwk)
    }
    _ => Err(format!("unsupported EC: {:?}", alg).into()),
  }
}

fn generate_rsa_jwk(alg: Algorithm) -> Result<JwkSQLRow, Box<dyn std::error::Error>> {
  let (alg_name, key_size) = match alg {
    Algorithm::RS256 => ("RS256".to_owned(), 2048),
    Algorithm::RS384 => ("RS384".to_owned(), 3072),
    Algorithm::RS512 => ("RS512".to_owned(), 4096),
    _ => return Err(format!("unsupported RSA: {:?}", alg).into()),
  };

  let mut rng = rand::thread_rng();

  let private_key = RsaPrivateKey::new(&mut rng, key_size)?;
  let public_key = RsaPublicKey::from(&private_key);

  let primes = private_key.primes();
  let p = &primes[0];
  let q = &primes[1];

  let dp = private_key.d() % (p - 1u8);
  let dq = private_key.d() % (q - 1u8);
  let qi = q.mod_inverse(p).ok_or_else(|| "inverse should exist")?;
  let (_, qi_bytes) = qi.to_bytes_be();

  let n = BASE64_URL_SAFE_NO_PAD.encode(&public_key.n().to_bytes_be());
  let e = BASE64_URL_SAFE_NO_PAD.encode(&public_key.e().to_bytes_be());
  let d = BASE64_URL_SAFE_NO_PAD.encode(&private_key.d().to_bytes_be());
  let p = BASE64_URL_SAFE_NO_PAD.encode(&p.to_bytes_be());
  let q = BASE64_URL_SAFE_NO_PAD.encode(&q.to_bytes_be());
  let dp = BASE64_URL_SAFE_NO_PAD.encode(&dp.to_bytes_be());
  let dq = BASE64_URL_SAFE_NO_PAD.encode(&dq.to_bytes_be());
  let qi = BASE64_URL_SAFE_NO_PAD.encode(&qi_bytes);

  let mut jwk_sql_row = JwkSQLRow::default();
  jwk_sql_row.alg = alg_name;
  jwk_sql_row.kty = "RSA".to_owned();
  jwk_sql_row.r#use = Some("sig".to_owned());
  jwk_sql_row.key_ops = Some("[\"sign\",\"verify\"]".to_owned());

  jwk_sql_row.n = Some(n);
  jwk_sql_row.e = Some(e);
  jwk_sql_row.d = Some(d);
  jwk_sql_row.p = Some(p);
  jwk_sql_row.q = Some(q);
  jwk_sql_row.dp = Some(dp);
  jwk_sql_row.dq = Some(dq);
  jwk_sql_row.qi = Some(qi);

  Ok(jwk_sql_row)
}

fn generate_ed_jwk() -> Result<JwkSQLRow, Box<dyn Error>> {
  use ed25519_dalek::SigningKey;

  let signing_key = SigningKey::generate(&mut rand::thread_rng());
  let verifying_key = signing_key.verifying_key();

  // Encode private (d) and public (x) parts
  let d = BASE64_URL_SAFE_NO_PAD.encode(signing_key.to_bytes());
  let x = BASE64_URL_SAFE_NO_PAD.encode(verifying_key.to_bytes());

  // Construct JWK row
  let mut jwk = JwkSQLRow::default();
  jwk.alg = "EdDSA".to_owned();
  jwk.kty = "OKP".to_owned(); // Octet Key Pair
  jwk.r#use = Some("sig".to_owned());
  jwk.key_ops = Some("[\"sign\",\"verify\"]".to_owned());
  jwk.crv = Some("Ed25519".to_owned());
  jwk.d = Some(d);
  jwk.x = Some(x);

  Ok(jwk)
}

pub fn generate_jwk(alg: Algorithm) -> Result<JwkSQLRow, Box<dyn std::error::Error>> {
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

fn encoding_key_hmac(jwk_sql_row: &JwkSQLRow) -> Result<EncodingKey, Box<dyn Error>> {
  let secret = BASE64_URL_SAFE_NO_PAD.decode(jwk_sql_row.k.clone().unwrap_or_default())?;
  Ok(EncodingKey::from_secret(secret.as_ref()))
}

fn encoding_key_es256(jwk_sql_row: &JwkSQLRow) -> Result<EncodingKey, Box<dyn Error>> {
  let d = jwk_sql_row
    .d
    .as_ref()
    .ok_or_else(|| "missing private key component 'd' for ES256")?;
  let d_bytes = BASE64_URL_SAFE_NO_PAD.decode(d)?;
  let secret_key: p256::elliptic_curve::SecretKey<p256::NistP256> =
    p256::elliptic_curve::SecretKey::from_slice(&d_bytes)
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
  Ok(EncodingKey::from_ec_der(
    P256EncodePrivateKey::to_pkcs8_der(&secret_key)?.as_bytes(),
  ))
}

fn encoding_key_es384(jwk_sql_row: &JwkSQLRow) -> Result<EncodingKey, Box<dyn Error>> {
  let d = jwk_sql_row
    .d
    .as_ref()
    .ok_or_else(|| "missing private key component 'd' for ES384")?;
  let d_bytes = BASE64_URL_SAFE_NO_PAD.decode(d)?;
  let secret_key: p384::elliptic_curve::SecretKey<p384::NistP384> =
    p384::elliptic_curve::SecretKey::from_slice(&d_bytes)
      .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
  Ok(EncodingKey::from_ec_der(
    P384EncodePrivateKey::to_pkcs8_der(&secret_key)?.as_bytes(),
  ))
}

fn encoding_key_rsa(jwk_sql_row: &JwkSQLRow) -> Result<EncodingKey, Box<dyn Error>> {
  use num_bigint_dig::BigUint;

  let n_b64 = jwk_sql_row.n.as_ref().ok_or("missing 'n' for RSA")?;
  let e_b64 = jwk_sql_row.e.as_ref().ok_or("missing 'e' for RSA")?;
  let d_b64 = jwk_sql_row.d.as_ref().ok_or("missing 'd' for RSA")?;
  let p_b64 = jwk_sql_row.p.as_ref().ok_or("missing 'p' for RSA")?;
  let q_b64 = jwk_sql_row.q.as_ref().ok_or("missing 'q' for RSA")?;

  let n = BigUint::from_bytes_be(&BASE64_URL_SAFE_NO_PAD.decode(n_b64)?);
  let e = BigUint::from_bytes_be(&BASE64_URL_SAFE_NO_PAD.decode(e_b64)?);
  let d = BigUint::from_bytes_be(&BASE64_URL_SAFE_NO_PAD.decode(d_b64)?);
  let p = BigUint::from_bytes_be(&BASE64_URL_SAFE_NO_PAD.decode(p_b64)?);
  let q = BigUint::from_bytes_be(&BASE64_URL_SAFE_NO_PAD.decode(q_b64)?);

  let rsa_key = RsaPrivateKey::from_components(n, e, d, vec![p, q])?;

  Ok(EncodingKey::from_rsa_der(
    RSAEncodePrivateKey::to_pkcs1_der(&rsa_key)?.as_bytes(),
  ))
}

fn encoding_key_ed(jwk_sql_row: &JwkSQLRow) -> Result<EncodingKey, Box<dyn Error>> {
  use ed25519_dalek::{SigningKey, pkcs8::EncodePrivateKey};

  let d = jwk_sql_row
    .d
    .as_ref()
    .ok_or_else(|| "missing private key component 'd' for EdDSA")?;
  let d_bytes = BASE64_URL_SAFE_NO_PAD.decode(d)?;

  let signing_key = SigningKey::from_bytes(
    &d_bytes
      .try_into()
      .map_err(|e| format!("invalid bytes: {:?}", e))?,
  );

  let der = signing_key.to_pkcs8_der()?;

  Ok(EncodingKey::from_ed_der(der.as_bytes()))
}

pub fn to_encoding_key(jwk_sql_row: &JwkSQLRow) -> Result<EncodingKey, Box<dyn Error>> {
  let alg = Algorithm::from_str(jwk_sql_row.alg.as_str())?;

  match alg {
    Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => encoding_key_hmac(jwk_sql_row),
    Algorithm::ES256 => encoding_key_es256(jwk_sql_row),
    Algorithm::ES384 => encoding_key_es384(jwk_sql_row),
    Algorithm::RS256
    | Algorithm::RS384
    | Algorithm::RS512
    | Algorithm::PS256
    | Algorithm::PS384
    | Algorithm::PS512 => encoding_key_rsa(jwk_sql_row),
    Algorithm::EdDSA => encoding_key_ed(jwk_sql_row),
  }
}

pub fn is_public_key_op(key_op: &String) -> bool {
  match key_op.as_str() {
    "verify" | "encrypt" | "wrapKey" => true,
    _ => false,
  }
}
