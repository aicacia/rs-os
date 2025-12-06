use std::io;

use uuid::Uuid;

pub const INCREMENTAL_CHUNK_TYPE: &str = "incremental";
pub const SNAPSHOT_CHUNK_TYPE: &str = "snapshot";

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum ChunkType {
  Incremental,
  Snapshot,
}

impl ChunkType {
  pub fn as_str(&self) -> &str {
    match self {
      Self::Incremental => INCREMENTAL_CHUNK_TYPE,
      Self::Snapshot => SNAPSHOT_CHUNK_TYPE,
    }
  }

  pub fn to_byte(&self) -> u8 {
    match self {
      Self::Incremental => 0,
      Self::Snapshot => 1,
    }
  }

  pub fn from_byte(byte: u8) -> io::Result<Self> {
    match byte {
      0 => Ok(Self::Incremental),
      1 => Ok(Self::Snapshot),
      _ => Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("invalid chunk type byte: {}", byte),
      )),
    }
  }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct StorageKey {
  pub uuid: Uuid,
  pub r#type: ChunkType,
  pub id: [u8; 32],
}

impl TryFrom<&[u8]> for StorageKey {
  type Error = io::Error;

  fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
    if bytes.len() != Self::BYTE_LEN {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("expected {} bytes, got {}", Self::BYTE_LEN, bytes.len()),
      ));
    }

    let uuid = Uuid::from_slice(&bytes[..16]).map_err(io::Error::other)?;
    let r#type = ChunkType::from_byte(bytes[16])?;

    let mut id = [0u8; 32];
    id.copy_from_slice(&bytes[17..Self::BYTE_LEN]);

    Ok(StorageKey { uuid, r#type, id })
  }
}

impl StorageKey {
  pub const BYTE_LEN: usize = 49;

  pub fn new(uuid: Uuid, r#type: ChunkType, id: [u8; 32]) -> Self {
    Self { uuid, r#type, id }
  }

  pub fn new_incremental(uuid: Uuid, id: [u8; 32]) -> Self {
    Self::new(uuid, ChunkType::Incremental, id)
  }

  pub fn new_snapshot(uuid: Uuid, id: [u8; 32]) -> Self {
    Self::new(uuid, ChunkType::Snapshot, id)
  }

  #[inline]
  pub fn as_bytes(&self) -> &[u8; Self::BYTE_LEN] {
    unsafe { &*(self as *const StorageKey as *const [u8; Self::BYTE_LEN]) }
  }

  pub fn to_bytes(&self) -> [u8; Self::BYTE_LEN] {
    *self.as_bytes()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_round_trip() {
    let original = StorageKey::new_incremental(
      Uuid::now_v7(),
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
