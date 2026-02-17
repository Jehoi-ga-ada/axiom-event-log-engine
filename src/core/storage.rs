use crate::core::StorageError;

pub trait LogStore: Send + Sync {
    fn append(&mut self, data: &[u8]) -> Result<u64, StorageError>;
    fn sync(&self) -> Result<(), StorageError>;
}