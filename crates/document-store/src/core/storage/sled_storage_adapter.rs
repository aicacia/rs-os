use std::{io, path::Path};

use crate::core::storage::storage_adapter::StorageAdapter;

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
    let db = sled::Config::default()
      .path(path)
      .open()
      .map_err(io::Error::other)?;

    Ok(Self { db })
  }
}

impl StorageAdapter for SledStorageAdapter {
  fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
    let value_bytes_optional = self.db.get(key).map_err(io::Error::other)?;

    match value_bytes_optional {
      Some(value_bytes) => Ok(Some(value_bytes.to_vec())),
      None => Ok(None),
    }
  }

  fn set(&self, key: &[u8], value: &[u8]) -> io::Result<()> {
    self.db.insert(key, value).map_err(io::Error::other)?;
    Ok(())
  }

  fn delete(&self, key: &[u8]) -> io::Result<()> {
    self.db.remove(key).map_err(io::Error::other)?;
    Ok(())
  }

  fn search<F>(&self, prefix: &[u8], mut f: F) -> io::Result<()>
  where
    F: FnMut((Vec<u8>, Vec<u8>)) -> io::Result<()>,
  {
    for result in self.db.range(prefix..) {
      let (k, v) = result.map_err(io::Error::other)?;

      if !k.starts_with(prefix) {
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
