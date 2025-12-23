use super::FSEntry::{FSEntry, FSEntryBase, Metadata};
use super::Chunk::Chunk;
use fuse::{FileType, FileAttr};
use time::Timespec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry{
    base: FSEntryBase,
    pub size: u64,
    chunks: Vec<Chunk>,
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

    fn make_file_attr(&self) -> FileAttr {
        let meta = self.get_metadata();
        let size = self.size;
        let blocks = ((size + super::FSEntry::BLOCK_SIZE - 1) / super::FSEntry::BLOCK_SIZE) as u64;
        FileAttr {
            ino: self.get_inode(),
            size,
            blocks,
            atime: Timespec::new(meta.accessed_at as i64, 0),
            mtime: Timespec::new(meta.modified_at as i64, 0),
            ctime: Timespec::new(meta.changed_at as i64, 0),
            crtime: Timespec::new(meta.created_at as i64, 0),
            kind: FileType::RegularFile,
            perm: meta.permissions,
            nlink: 1,
            uid: meta.uid,
            gid: meta.gid,
            rdev: meta.rdev,
            flags: meta.flags,
        }
    }
}

impl FileEntry {
    pub fn new(base: FSEntryBase, size: u64, chunks: Vec<Chunk>, checksum: [u8; 32]) -> Self {
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

    pub fn get_chunks(&self) -> Vec<Chunk> {
        self.chunks.clone()
    }

    pub fn get_chunks_in_range(&self, offset: usize, length: usize) -> Vec<Chunk> {
        self.chunks[offset..offset + length].to_vec()
    }

    pub fn set_chunks(&mut self, chunks: Vec<Chunk>) {
        self.chunks = chunks;
    }

    pub fn set_chunks_at_offset(&mut self, chunks: Vec<Chunk>, offset: usize) {
        for (i, chunk) in chunks.iter().enumerate() {
            if offset + i < self.chunks.len() {
                self.chunks[offset + i] = chunk.clone();
            } else {
                self.chunks.push(chunk.clone());
            }
        }
    }

}