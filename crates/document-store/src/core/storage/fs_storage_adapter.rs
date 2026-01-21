use std::{
  fs, io,
  path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::core::storage::{
  storage_adapter::StorageAdapter,
  storage_key::{ChunkType, StorageKey},
};

pub struct FSStorageAdapter {
  path: PathBuf,
}

impl<'a> TryFrom<&'a Path> for FSStorageAdapter {
  type Error = io::Error;

  fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
    fs::create_dir_all(&path)?;

    Ok(Self {
      path: path.to_path_buf(),
    })
  }
}

impl FSStorageAdapter {
  fn key_path(&self, key: &StorageKey) -> PathBuf {
    let uuid_dir = self.path.join(key.uuid.to_string());
    let filename = format!("{}.{}", hex::encode(key.id), key.r#type.as_str());
    uuid_dir.join(filename)
  }

  fn uuid_dir(&self, uuid: Uuid) -> PathBuf {
    self.path.join(uuid.to_string())
  }
}

impl StorageAdapter for FSStorageAdapter {
  fn get(&self, key: &StorageKey) -> io::Result<Option<Vec<u8>>> {
    let path = self.key_path(key);
    match fs::read(&path) {
      Ok(data) => Ok(Some(data)),
      Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
      Err(e) => Err(e),
    }
  }

  fn set(&self, key: &StorageKey, value: &[u8]) -> io::Result<()> {
    let path = self.key_path(key);
    let dir = path
      .parent()
      .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid key path"))?;
    fs::create_dir_all(dir)?;
    fs::write(&path, value)?;
    Ok(())
  }

  fn delete(&self, key: &StorageKey) -> io::Result<()> {
    let path = self.key_path(key);
    match fs::remove_file(&path) {
      Ok(()) => Ok(()),
      Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
      Err(e) => Err(e),
    }
  }

  fn search<F>(&self, uuid: Uuid, mut f: F) -> io::Result<()>
  where
    F: FnMut((Vec<u8>, Vec<u8>)) -> io::Result<()>,
  {
    let uuid_dir = self.uuid_dir(uuid);
    match fs::read_dir(&uuid_dir) {
      Ok(entries) => {
        for entry in entries {
          let entry = entry?;
          let path = entry.path();
          if path.is_file() {
            let data = fs::read(&path)?;
            let filename = path
              .file_name()
              .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid filename"))?
              .to_string_lossy();

            let parts: Vec<&str> = filename.split('.').collect();
            if parts.len() != 2 {
              continue;
            }

            let id_hex = parts[0];
            let type_str = parts[1];

            let id = match hex::decode(id_hex) {
              Ok(bytes) if bytes.len() == 32 => {
                let mut id = [0u8; 32];
                id.copy_from_slice(&bytes);
                id
              }
              _ => continue,
            };

            // Parse chunk type
            let chunk_type = match type_str {
              "incremental" => ChunkType::Incremental,
              "snapshot" => ChunkType::Snapshot,
              _ => continue,
            };

            // Reconstruct StorageKey bytes
            let key = StorageKey::new(uuid, chunk_type, id);
            let key_bytes = key.to_bytes().to_vec();

            f((key_bytes, data))?;
          }
        }
        Ok(())
      }
      Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
      Err(e) => Err(e),
    }
  }
}
