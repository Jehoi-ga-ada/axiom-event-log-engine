pub mod storage;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("Segment full")]
    SegmentFull,
    #[error("Checksum mismatch at offset {0}")]
    Corruption(u64)
}