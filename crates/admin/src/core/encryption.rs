use rand::Rng;

use crate::config::AppConfig;

pub fn random_bytes(size: usize) -> Vec<u8> {
  let mut bytes = vec![0; size];
  rand::thread_rng().fill(bytes.as_mut_slice());
  bytes
}

pub fn encrypt_password(config: &AppConfig, input: &str) -> argon2::Result<String> {
  argon2::hash_encoded(
    input.as_bytes(),
    random_bytes(config.password.salt_length).as_slice(),
    &argon2_config(config),
  )
}

fn argon2_config<'a>(config: &AppConfig) -> argon2::Config<'a> {
  argon2::Config {
    variant: argon2::Variant::Argon2id,
    hash_length: config.password.hash_length,
    lanes: config.password.parallelism,
    mem_cost: config.password.memory_mib * 1024,
    time_cost: config.password.iterations,
    ..Default::default()
  }
}
