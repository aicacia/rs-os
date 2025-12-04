use std::io;

pub const INCREMENTAL_CHUNK_TYPE: &str = "incremental";
pub const SNAPSHOT_CHUNK_TYPE: &str = "snapshot";
pub const SYNC_STATE_CHUNK_TYPE: &str = "sync-state";

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum ChunkType {
  Incremental,
  Snapshot,
  SyncState,
}

impl ChunkType {
  pub fn as_str(&self) -> &str {
    match self {
      Self::Incremental => INCREMENTAL_CHUNK_TYPE,
      Self::Snapshot => SNAPSHOT_CHUNK_TYPE,
      Self::SyncState => SYNC_STATE_CHUNK_TYPE,
    }
  }

  pub fn to_byte(&self) -> u8 {
    match self {
      Self::Incremental => 0,
      Self::Snapshot => 1,
      Self::SyncState => 2,
    }
  }

  pub fn from_byte(byte: u8) -> io::Result<Self> {
    match byte {
      0 => Ok(Self::Incremental),
      1 => Ok(Self::Snapshot),
      2 => Ok(Self::SyncState),
      _ => Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("invalid chunk type byte: {}", byte),
      )),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct StorageKey {
  pub key: String,
  pub r#type: ChunkType,
  pub id: [u8; 32],
}

impl TryFrom<&[u8]> for StorageKey {
  type Error = io::Error;

  fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
    if bytes.len() < 34 {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("expected at least 34 bytes, got {}", bytes.len()),
      ));
    }

    // Read key length (first 2 bytes, big-endian u16)
    let key_len = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;

    if bytes.len() != 2 + key_len + 1 + 32 {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
          "expected {} bytes, got {}",
          2 + key_len + 1 + 32,
          bytes.len()
        ),
      ));
    }

    let key = String::from_utf8(bytes[2..2 + key_len].to_vec())
      .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let r#type = ChunkType::from_byte(bytes[2 + key_len])?;

    let mut id = [0u8; 32];
    id.copy_from_slice(&bytes[2 + key_len + 1..]);

    Ok(StorageKey { key, r#type, id })
  }
}

impl StorageKey {
  pub fn new(key: String, r#type: ChunkType, id: [u8; 32]) -> Self {
    Self { key, r#type, id }
  }

  pub fn new_incremental(key: String, id: [u8; 32]) -> Self {
    Self::new(key, ChunkType::Incremental, id)
  }

  pub fn new_snapshot(key: String, id: [u8; 32]) -> Self {
    Self::new(key, ChunkType::Snapshot, id)
  }

  pub fn new_sync_state(key: String, id: [u8; 32]) -> Self {
    Self::new(key, ChunkType::SyncState, id)
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    let key_bytes = self.key.as_bytes();
    let key_len = key_bytes.len() as u16;
    let mut bytes = Vec::with_capacity(2 + key_bytes.len() + 1 + 32);

    // Write key length (2 bytes, big-endian)
    bytes.extend_from_slice(&key_len.to_be_bytes());
    // Write key
    bytes.extend_from_slice(key_bytes);
    // Write type
    bytes.push(self.r#type.to_byte());
    // Write id
    bytes.extend_from_slice(&self.id);

    bytes
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_round_trip() {
    let original = StorageKey::new_incremental(
      "test-key".to_string(),
      [
        0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
        0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
        0xde, 0xf0,
      ],
    );

    let bytes = original.to_bytes();
    let parsed = StorageKey::try_from(bytes.as_slice()).unwrap();

    assert_eq!(original, parsed);
  }

  #[test]
  fn test_round_trip_long_key() {
    let long_key = "this-is-a-much-longer-key-that-can-be-any-arbitrary-size".to_string();
    let original = StorageKey::new_snapshot(
      long_key,
      [
        0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
        0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
        0xde, 0xf0,
      ],
    );

    let bytes = original.to_bytes();
    let parsed = StorageKey::try_from(bytes.as_slice()).unwrap();

    assert_eq!(original, parsed);
  }
}
