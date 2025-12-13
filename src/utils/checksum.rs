
pub fn calculate_checksum(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut checksum = [0u8; 32];
    checksum.copy_from_slice(&result);
    return checksum
}