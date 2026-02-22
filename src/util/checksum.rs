use crc32fast::Hasher;

pub fn calculate(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

pub fn validate(data: &[u8], expected: u32) -> bool {
    calculate(data) == expected
}