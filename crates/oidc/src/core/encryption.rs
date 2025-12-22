use rand::Rng;

pub fn random_bytes(size: usize) -> Vec<u8> {
  let mut bytes = Vec::with_capacity(size);
  bytes.resize(size, 0);
  rand::thread_rng().fill(bytes.as_mut_slice());
  bytes
}

pub fn verify_password(input: &str, encrypted_password: &str) -> argon2::Result<bool> {
  argon2::verify_encoded(encrypted_password, input.as_bytes())
}
