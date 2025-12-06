use sha2::{Digest, Sha256};

pub fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
  let mut hasher = Sha256::new();
  hasher.update(bytes);
  let result = hasher.finalize();

  let mut hash = [0u8; 32];
  hash.copy_from_slice(&result);

  hash
}

pub fn hash_changes<T>(changes: &[T]) -> [u8; 32]
where
  T: AsRef<[u8]>,
{
  let mut hasher = Sha256::new();

  for change in changes {
    hasher.update(change.as_ref());
  }

  let result = hasher.finalize();

  let mut hash = [0u8; 32];
  hash.copy_from_slice(&result);

  hash
}
