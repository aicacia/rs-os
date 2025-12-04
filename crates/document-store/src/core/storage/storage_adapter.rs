use std::io;

pub trait StorageAdapter {
  fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>>;
  fn set(&self, key: &[u8], value: &[u8]) -> io::Result<()>;
  fn delete(&self, key: &[u8]) -> io::Result<()>;
  fn search<F>(&self, prefix: &[u8], f: F) -> io::Result<()>
  where
    F: FnMut((Vec<u8>, Vec<u8>)) -> io::Result<()>;
  fn flush(&self) -> io::Result<()> {
    Ok(())
  }
}
