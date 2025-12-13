
//a Filesystem primitive to represent a chunk, not actual storage, just a lookup metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    pub file_id: u128,
    pub idx: u64,
    pub id: u128,        // routing identity
    pub size: u64,
    pub hash: [u8; 32],  // integrity checksum
}

impl Chunk {
    pub fn new_from_data(file_id: u128, idx: u64, data: &[u8]) -> Self {
        let size = data.len() as u64;
        use crate::utils::checksum::calculate_checksum;
        use sha2::{Sha256, Digest};

        let hash = calculate_checksum(data);
        let id = Self::make_chunk_id(file_id, idx);
        Chunk {
            file_id,
            idx,
            id,
            size,
            hash,
        }
    }

    fn make_chunk_id(file_id: u128, idx: u64) -> u128 {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(file_id.to_le_bytes());
        hasher.update(idx.to_le_bytes());
        let out = hasher.finalize();
        u128::from_le_bytes(out[0..16].try_into().unwrap())
    }
}