use super::FSEntry::{FSEntry, FSEntryBase, Metadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry{
    base: FSEntryBase,
    pub size: u64,
    chunks: Vec<u64>, // List of chunk IDs, not the actual chunks because stored on different nodes
    pub checksum: [u8; 32], // Example: SHA-256 checksum
}

impl FSEntry for FileEntry {
    fn get_name(&self) -> &str {
        &self.base.name
    }

    fn get_metadata(&self) -> &Metadata {
        &self.base.metadata
    }

    fn get_inode(&self) -> u64 {
        self.base.inode
    }
}

impl FileEntry {
    pub fn new(base: FSEntryBase, size: u64, chunks: Vec<u64>, checksum: [u8; 32]) -> Self {
        FileEntry {
            base,
            size,
            chunks,
            checksum,
        }
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn get_chunks(&self) -> Vec<u64> {
        self.chunks.clone()
    }

    pub fn get_chunks_in_range(&self, offset: usize, length: usize) -> Vec<u64> {
        self.chunks[offset..offset + length].to_vec()
    }

    pub fn set_chunks(&mut self, chunks: Vec<u64>) {
        self.chunks = chunks;
    }

    pub fn set_chunks_at_offset(&mut self, chunks: Vec<u64>, offset: usize) {
        for (i, &chunk_id) in chunks.iter().enumerate() {
            if offset + i < self.chunks.len() {
                self.chunks[offset + i] = chunk_id;
            } else {
                self.chunks.push(chunk_id);
            }
        }
    }


}