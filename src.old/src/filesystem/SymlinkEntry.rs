
use super::FSEntry::{FSEntry, FSEntryBase, Metadata};
use fuse::{FileType, FileAttr};
use time::Timespec;

pub struct SymlinkEntry {
    pub base: FSEntryBase,
    pub target: Box<dyn FSEntry>,
}

impl FSEntry for SymlinkEntry {
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
        let target_len = self.target.get_name().len() as u64;
        let blocks = if target_len > 0 { ((target_len + super::FSEntry::BLOCK_SIZE - 1) / super::FSEntry::BLOCK_SIZE) as u64 } else { 0 };
        FileAttr {
            ino: self.get_inode(),
            size: target_len,
            blocks,
            atime: Timespec::new(meta.accessed_at as i64, 0),
            mtime: Timespec::new(meta.modified_at as i64, 0),
            ctime: Timespec::new(meta.changed_at as i64, 0),
            crtime: Timespec::new(meta.created_at as i64, 0),
            kind: FileType::Symlink,
            perm: meta.permissions,
            nlink: 1,
            uid: meta.uid,
            gid: meta.gid,
            rdev: meta.rdev,
            flags: meta.flags,
        }
    }
}

impl SymlinkEntry {
    pub fn new(base: FSEntryBase, target: Box<dyn FSEntry>) -> Self {
        SymlinkEntry { base, target }
    }

    pub fn get_target(&self) -> &Box<dyn FSEntry> {
        &self.target
    }

    pub fn set_target(&mut self, target: Box<dyn FSEntry>) {
        self.target = target;
    }
}