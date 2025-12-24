use rand::Rng;

pub fn random_bytes(size: usize) -> Vec<u8> {
  let mut bytes = vec![0; size];
  rand::thread_rng().fill(bytes.as_mut_slice());
  bytes
}

pub fn verify_password(input: &str, encrypted_password: &str) -> argon2::Result<bool> {
  argon2::verify_encoded(encrypted_password, input.as_bytes())
}
