use std::io;

use uuid::Uuid;

use crate::core::storage::storage_key::StorageKey;

pub trait StorageAdapter {
  fn get(&self, key: &StorageKey) -> io::Result<Option<Vec<u8>>>;
  fn set(&self, key: &StorageKey, value: &[u8]) -> io::Result<()>;
  fn delete(&self, key: &StorageKey) -> io::Result<()>;
  fn search<F>(&self, uuid: Uuid, f: F) -> io::Result<()>
  where
    F: FnMut((Vec<u8>, Vec<u8>)) -> io::Result<()>;
  fn flush(&self) -> io::Result<()> {
    Ok(())
  }
}
