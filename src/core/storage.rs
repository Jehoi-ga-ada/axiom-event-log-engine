use crate::core::StorageError;

pub trait LogStore: Send + Sync {
    fn append(&mut self, data: &[u8]) -> Result<u64, StorageError>;
    fn sync(&self) -> Result<(), StorageError>;
    fn append_with_checksum(&mut self, data: &[u8], crc: u32) -> Result<u64, crate::core::StorageError>;
}