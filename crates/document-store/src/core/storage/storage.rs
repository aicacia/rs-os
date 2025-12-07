use std::{
  collections::{HashMap, HashSet},
  io,
};

use automerge::{ActorId, Automerge, ChangeHash};
use dashmap::DashMap;
use uuid::Uuid;

use crate::core::storage::{
  helper::{hash_bytes, hash_changes},
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

  fn should_save_document(&self, document_id: Uuid, document: &Automerge) -> bool {
    let old_heads = match self.stored_heads.get(&document_id) {
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
    document_id: Uuid,
    document: &Automerge,
    chunk_info: &mut HashMap<StorageKey, usize>,
    stored_heads: &mut Vec<ChangeHash>,
  ) -> io::Result<()> {
    let bytes = document.save();
    let new_heads = document.get_heads();
    let snapshot_hash = hash_changes(&new_heads);
    let old_keys: HashSet<_> = chunk_info
      .keys()
      .filter_map(|storage_key| {
        if &storage_key.id != &snapshot_hash {
          Some(storage_key.clone())
        } else {
          None
        }
      })
      .collect();
    let storage_key = StorageKey::new(document_id, ChunkType::Snapshot, snapshot_hash);

    self.storage_adapter.set(&storage_key, &bytes)?;

    for old_key in &old_keys {
      self.storage_adapter.delete(&old_key)?;
    }

    for old_key in old_keys {
      chunk_info.remove(&old_key);
    }
    chunk_info.insert(storage_key, bytes.len());

    *stored_heads = new_heads;

    Ok(())
  }

  fn save_incremental(
    &self,
    document_id: Uuid,
    document: &Automerge,
    chunk_info: &mut HashMap<StorageKey, usize>,
    stored_heads: &mut Vec<ChangeHash>,
  ) -> io::Result<()> {
    let bytes = document.save_after(&stored_heads);

    if bytes.is_empty() {
      return Ok(());
    }

    let snapshot_hash = hash_bytes(&bytes);
    let storage_key = StorageKey::new(document_id, ChunkType::Incremental, snapshot_hash);

    self.storage_adapter.set(&storage_key, &bytes)?;

    chunk_info.insert(storage_key.clone(), bytes.len());

    *stored_heads = document.get_heads();

    Ok(())
  }

  pub fn save_document(&self, document_id: Uuid, document: &Automerge) -> io::Result<()> {
    if !self.should_save_document(document_id, document) {
      return Ok(());
    }

    let mut chunk_info = self.chunk_infos.entry(document_id).or_default();
    let mut stored_heads = self.stored_heads.entry(document_id).or_default();

    if self.should_compact(chunk_info.value()) {
      self.save_compact(
        document_id,
        document,
        chunk_info.value_mut(),
        stored_heads.value_mut(),
      )
    } else {
      self.save_incremental(
        document_id,
        document,
        chunk_info.value_mut(),
        stored_heads.value_mut(),
      )
    }
  }

  pub fn load_document(&self, document_id: Uuid) -> io::Result<Option<Automerge>> {
    let mut bytes = Vec::new();
    let mut chunk_info = HashMap::new();

    self.storage_adapter.search(document_id, |(k, mut v)| {
      let storage_key = StorageKey::try_from(k.as_slice())?;

      chunk_info.insert(storage_key, v.len());
      bytes.append(&mut v);

      Ok(())
    })?;

    if bytes.is_empty() {
      return Ok(None);
    }

    self.chunk_infos.insert(document_id, chunk_info);

    let mut document: Automerge =
      Automerge::new().with_actor(ActorId::from(document_id.as_bytes()));

    document
      .load_incremental(&bytes)
      .map_err(io::Error::other)?;

    self.stored_heads.insert(document_id, document.get_heads());

    Ok(Some(document))
  }

  pub fn remove_document(self, document_id: Uuid) -> io::Result<()> {
    let mut storage_keys = Vec::new();

    self.storage_adapter.search(document_id, |(k, _)| {
      storage_keys.push(StorageKey::try_from(k.as_slice())?);
      Ok(())
    })?;

    for storage_key in storage_keys {
      self.storage_adapter.delete(&storage_key)?;
    }

    Ok(())
  }

  pub fn flush(&self) -> io::Result<()> {
    self.storage_adapter.flush()
  }
}
