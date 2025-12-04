use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  ops::Deref,
  path::Path,
  sync::Arc,
};

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use dashmap::{DashMap, mapref::entry::Entry};
use once_cell::sync::Lazy;
use os_api::{HttpError, INTERNAL_ERROR};
use tokio::fs;

use crate::{
  core::storage::{sled_storage_adapter::SledStorageAdapter, storage::Storage},
  router::ws::constants::DATA_PATH_DOCUMENTS,
};

pub struct SharedStorage {
  id: uuid::Uuid,
  inner: Arc<Storage<SledStorageAdapter>>,
  key: u64,
}

static STORAGE_REGISTRY_SLED: Lazy<DashMap<u64, Arc<Storage<SledStorageAdapter>>>> =
  Lazy::new(DashMap::new);

impl SharedStorage {
  fn storage_key(aud: &str, sub: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    aud.hash(&mut hasher);
    sub.hash(&mut hasher);
    hasher.finish()
  }

  pub fn id(&self) -> uuid::Uuid {
    self.id
  }

  pub fn peer_id(&self) -> String {
    format!("server-{}", self.key)
  }

  pub async fn get(base_path: &Path, aud: &str, sub: &str) -> Result<Self, HttpError> {
    let key = Self::storage_key(aud, sub);
    let inner = match STORAGE_REGISTRY_SLED.entry(key) {
      Entry::Occupied(existing) => existing.get().clone(),
      Entry::Vacant(vacant) => {
        let storage_path = base_path
          .join(DATA_PATH_DOCUMENTS)
          .join(BASE64_URL_SAFE_NO_PAD.encode(aud.as_bytes()))
          .join(BASE64_URL_SAFE_NO_PAD.encode(sub.as_bytes()));

        if let Err(err) = fs::create_dir_all(&storage_path).await {
          log::error!(
            "failed to create document store storage path {}: {}",
            storage_path.display(),
            err
          );
          return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
        }

        let storage_adapter = match SledStorageAdapter::try_from(storage_path.as_path()) {
          Ok(adapter) => adapter,
          Err(err) => {
            log::error!(
              "failed to create document store storage adapter for path {}: {}",
              storage_path.display(),
              err
            );
            return Err(HttpError::internal_error().with_application_error(INTERNAL_ERROR));
          }
        };

        let storage = Storage::new(storage_adapter);
        let inner = Arc::new(storage);
        vacant.insert(inner.clone());
        inner
      }
    };

    Ok(SharedStorage {
      id: uuid::Uuid::now_v7(),
      inner,
      key,
    })
  }
}

impl Drop for SharedStorage {
  fn drop(&mut self) {
    if Arc::strong_count(&self.inner) == 2 {
      STORAGE_REGISTRY_SLED.remove(&self.key);
    }
  }
}

impl Deref for SharedStorage {
  type Target = Storage<SledStorageAdapter>;

  fn deref(&self) -> &Self::Target {
    self.inner.as_ref()
  }
}
