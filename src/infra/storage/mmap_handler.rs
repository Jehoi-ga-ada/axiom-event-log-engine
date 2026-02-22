use std::fs::{File, OpenOptions};
use memmap2::MmapMut;
use crate::core::storage::LogStore;
use crate::core::StorageError;

pub struct MmapStore{
    _file: File,
    mmap: MmapMut,
    write_ptr: usize,
    capacity: usize,
}

impl MmapStore {
    pub fn new(path: &str, size: usize) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        file.set_len(size as u64)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        Ok(Self {
            _file: file,
            mmap,
            write_ptr: 0,
            capacity: size,
        })        
    }
}

impl LogStore for MmapStore {
    fn append(&mut self, data: &[u8]) -> Result<u64, StorageError> {
        let len = data.len();
        
        if self.write_ptr + len > self.capacity {
            return Err(StorageError::SegmentFull);
        }

        self.mmap[self.write_ptr..self.write_ptr + len].copy_from_slice(data);
        
        let offset = self.write_ptr as u64;
        self.write_ptr += len;

        Ok(offset)
    }

    fn append_with_checksum(&mut self, data: &[u8], crc: u32) -> Result<u64, StorageError> {
        let payload_len = data.len();
        let total_len = 4 + 4 + payload_len; // [Header: 8B] + [Data: NB]

        if self.write_ptr + total_len > self.capacity {
            return Err(StorageError::SegmentFull);
        }

        let start = self.write_ptr;

        // 1. Write Length (Big Endian)
        let len_bytes = (payload_len as u32).to_be_bytes();
        self.mmap[start..start + 4].copy_from_slice(&len_bytes);

        // 2. Write Checksum (Big Endian)
        let crc_bytes = crc.to_be_bytes();
        self.mmap[start + 4..start + 8].copy_from_slice(&crc_bytes);

        // 3. Write Data
        self.mmap[start + 8..start + total_len].copy_from_slice(data);

        let offset = self.write_ptr as u64;
        self.write_ptr += total_len;

        Ok(offset)
    }

    fn sync(&self) -> Result<(), StorageError> {
        self.mmap.flush().map_err(StorageError::Io)
    }
}