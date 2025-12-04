use std::{
  collections::{HashMap, HashSet},
  io,
};

use automerge::{ActorId, Automerge, ChangeHash, sync::State};
use dashmap::DashMap;
use uuid::Uuid;

use crate::core::storage::{
  storage_adapter::StorageAdapter,
  storage_key::{ChunkType, StorageKey},
};

pub struct Storage<SA>
where
  SA: StorageAdapter,
{
  storage_adapter: SA,
  // TODO: use LRU cache
  chunk_infos: DashMap<Uuid, HashMap<StorageKey, usize>>,
  // TODO: use LRU cache
  stored_heads: DashMap<Uuid, Vec<ChangeHash>>,
}

impl<SA> From<SA> for Storage<SA>
where
  SA: StorageAdapter,
{
  fn from(storage_adapter: SA) -> Self {
    Self::new(storage_adapter)
  }
}

impl<SA> Storage<SA>
where
  SA: StorageAdapter,
{
  pub fn new(storage_adapter: SA) -> Self {
    Self {
      storage_adapter,
      chunk_infos: DashMap::default(),
      stored_heads: DashMap::default(),
    }
  }

  fn should_save_document(&self, uuid: Uuid, document: &Automerge) -> bool {
    let old_heads = match self.stored_heads.get(&uuid) {
      Some(old_heads) => old_heads,
      None => {
        log::debug!("no cached heads should save");
        return true;
      }
    };

    let new_heads = document.get_heads();

    if new_heads.len() != old_heads.len() {
      log::debug!("head count mismatch should save");
      return true;
    }

    for i in 0..new_heads.len() {
      if &new_heads[i] != &old_heads[i] {
        log::debug!("heads do not match should save");
        return true;
      }
    }

    log::debug!("cache matches document should not save");
    false
  }

  fn should_compact(&self, chunk_info: &HashMap<StorageKey, usize>) -> bool {
    let mut snapshot_size = 0;
    let mut incremental_size = 0;

    for (key, size) in chunk_info {
      match key.r#type {
        ChunkType::Incremental => incremental_size += size,
        ChunkType::Snapshot => snapshot_size += size,
        _ => {}
      }
    }

    if snapshot_size < 1024 {
      log::debug!("snapshot less than 1K should compact");
      return true;
    }
    if incremental_size >= snapshot_size {
      log::debug!("incremental is greater than snapshot should compact");
      return true;
    }
    false
  }

  fn save_compact(
    &self,
    uuid: Uuid,
    document: &Automerge,
    chunk_info: &mut HashMap<StorageKey, usize>,
    stored_heads: &mut Vec<ChangeHash>,
  ) -> io::Result<()> {
    let bytes = document.save();
    let new_heads = document.get_heads();
    let snapshot_hash = hash_changes(&new_heads);
    let old_keys: HashSet<_> = chunk_info
      .keys()
      .filter_map(|key| {
        if &key.id != &snapshot_hash {
          Some(key.clone())
        } else {
          None
        }
      })
      .collect();
    let key = StorageKey::new(uuid, ChunkType::Snapshot, snapshot_hash);

    self.storage_adapter.set(key.as_bytes(), &bytes)?;

    for old_key in &old_keys {
      self.storage_adapter.delete(old_key.as_bytes())?;
    }

    for old_key in old_keys {
      chunk_info.remove(&old_key);
    }
    chunk_info.insert(key, bytes.len());

    *stored_heads = new_heads;

    Ok(())
  }

  fn save_incremental(
    &self,
    uuid: Uuid,
    document: &Automerge,
    chunk_info: &mut HashMap<StorageKey, usize>,
    stored_heads: &mut Vec<ChangeHash>,
  ) -> io::Result<()> {
    let bytes = document.save_after(&stored_heads);

    if bytes.is_empty() {
      return Ok(());
    }

    let snapshot_hash = hash_bytes(&bytes);
    let key = StorageKey::new(uuid, ChunkType::Incremental, snapshot_hash);

    self.storage_adapter.set(key.as_bytes(), &bytes)?;

    chunk_info.insert(key.clone(), bytes.len());

    *stored_heads = document.get_heads();

    Ok(())
  }

  pub fn save_document(&self, uuid: Uuid, document: &Automerge) -> io::Result<()> {
    if !self.should_save_document(uuid, document) {
      return Ok(());
    }

    let mut chunk_info = self.chunk_infos.entry(uuid).or_default();
    let mut stored_heads = self.stored_heads.entry(uuid).or_default();

    if self.should_compact(chunk_info.value()) {
      self.save_compact(
        uuid,
        document,
        chunk_info.value_mut(),
        stored_heads.value_mut(),
      )
    } else {
      self.save_incremental(
        uuid,
        document,
        chunk_info.value_mut(),
        stored_heads.value_mut(),
      )
    }
  }

  pub fn save_sync_state(&self, uuid: Uuid, id: [u8; 32], sync_state: State) -> io::Result<()> {
    let storage_key = StorageKey::new_sync_state(uuid, id);
    let bytes = sync_state.encode();

    self.storage_adapter.set(storage_key.as_bytes(), &bytes)
  }

  pub fn load_document(&self, uuid: Uuid) -> io::Result<(Automerge, bool)> {
    let mut bytes = Vec::new();
    let mut chunk_info = HashMap::new();

    self.storage_adapter.search(uuid.as_bytes(), |(k, mut v)| {
      let storage_key = StorageKey::try_from(k.as_slice())?;

      chunk_info.insert(storage_key, v.len());
      bytes.append(&mut v);

      Ok(())
    })?;

    self.chunk_infos.insert(uuid, chunk_info);

    let mut document: Automerge = Automerge::new().with_actor(ActorId::from(uuid.as_bytes()));

    let is_new = bytes.is_empty();

    if !is_new {
      document
        .load_incremental(&bytes)
        .map_err(io::Error::other)?;
    }

    self.stored_heads.insert(uuid, document.get_heads());

    Ok((document, is_new))
  }

  pub fn remove_document(self, uuid: &Uuid) -> io::Result<()> {
    let mut storage_keys = Vec::new();

    self.storage_adapter.search(uuid.as_bytes(), |(k, _)| {
      storage_keys.push(StorageKey::try_from(k.as_slice())?);
      Ok(())
    })?;

    for storage_key in storage_keys {
      self.storage_adapter.delete(storage_key.as_bytes())?;
    }

    Ok(())
  }

  pub fn flush(&self) -> io::Result<()> {
    self.storage_adapter.flush()
  }
}

fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
  use sha2::{Digest, Sha256};

  let mut hasher = Sha256::new();
  hasher.update(bytes);
  let result = hasher.finalize();

  let mut hash = [0u8; 32];
  hash.copy_from_slice(&result);

  hash
}

fn hash_changes<T>(changes: &[T]) -> [u8; 32]
where
  T: AsRef<[u8]>,
{
  use sha2::{Digest, Sha256};

  let mut hasher = Sha256::new();

  for change in changes {
    hasher.update(change.as_ref());
  }

  let result = hasher.finalize();

  let mut hash = [0u8; 32];
  hash.copy_from_slice(&result);

  hash
}
