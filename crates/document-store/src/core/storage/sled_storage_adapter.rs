use std::{fs, io, path::Path};

use uuid::Uuid;

use crate::core::storage::{storage_adapter::StorageAdapter, storage_key::StorageKey};

pub struct SledStorageAdapter {
  db: sled::Db,
}

impl From<sled::Db> for SledStorageAdapter {
  fn from(db: sled::Db) -> Self {
    Self { db }
  }
}

impl<'a> TryFrom<&'a Path> for SledStorageAdapter {
  type Error = io::Error;

  fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
    fs::create_dir_all(&path)?;

    let db = sled::Config::default()
      .path(path)
      .open()
      .map_err(io::Error::other)?;

    Ok(Self { db })
  }
}

impl StorageAdapter for SledStorageAdapter {
  fn get(&self, key: &StorageKey) -> io::Result<Option<Vec<u8>>> {
    let value_bytes_optional = self.db.get(key.as_bytes()).map_err(io::Error::other)?;

    match value_bytes_optional {
      Some(value_bytes) => Ok(Some(value_bytes.to_vec())),
      None => Ok(None),
    }
  }

  fn set(&self, key: &StorageKey, value: &[u8]) -> io::Result<()> {
    self
      .db
      .insert(key.as_bytes(), value)
      .map_err(io::Error::other)?;
    Ok(())
  }

  fn delete(&self, key: &StorageKey) -> io::Result<()> {
    self.db.remove(key.as_bytes()).map_err(io::Error::other)?;
    Ok(())
  }

  fn search<F>(&self, prefix: Uuid, mut f: F) -> io::Result<()>
  where
    F: FnMut((Vec<u8>, Vec<u8>)) -> io::Result<()>,
  {
    let prefix_bytes = prefix.as_bytes().as_ref();

    for result in self.db.range(prefix_bytes..) {
      let (k, v) = result.map_err(io::Error::other)?;

      if !k.starts_with(prefix_bytes) {
        continue;
      }

      f((k.to_vec(), v.to_vec()))?;
    }
    Ok(())
  }

  fn flush(&self) -> io::Result<()> {
    let _ = self.db.flush().map_err(io::Error::other);
    Ok(())
  }
}
