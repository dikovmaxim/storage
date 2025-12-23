use fuse::{FileType, FileAttr};
use time::Timespec;

pub trait FSEntry {
    fn get_name(&self) -> &str;
    fn get_metadata(&self) -> &Metadata;
    fn get_inode(&self) -> u64;
    fn make_file_attr(&self) -> FileAttr;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FSEntryBase {
    pub inode: u64,
    pub name: String,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub created_at: u64,
    pub modified_at: u64,
    pub accessed_at: u64,
    pub changed_at: u64,
    pub permissions: u16,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u32,
    pub flags: u32,
}

pub const BLOCK_SIZE: u64 = 4096;