use std::collections::HashMap;
use super::FSEntry::{FSEntry, FSEntryBase, Metadata};
use fuse::{FileType, FileAttr};
use time::Timespec;


pub struct DirEntry {
    pub base: FSEntryBase,
    pub children: HashMap<u64, Box<dyn FSEntry>>,
}

impl FSEntry for DirEntry {
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
        FileAttr {
            ino: self.get_inode(),
            size: 0,
            blocks: 0,
            atime: Timespec::new(meta.accessed_at as i64, 0),
            mtime: Timespec::new(meta.modified_at as i64, 0),
            ctime: Timespec::new(meta.changed_at as i64, 0),
            crtime: Timespec::new(meta.created_at as i64, 0),
            kind: FileType::Directory,
            perm: meta.permissions,
            nlink: 2,
            uid: meta.uid,
            gid: meta.gid,
            rdev: meta.rdev,
            flags: meta.flags,
        }
    }
}

impl DirEntry {
    pub fn new(base: FSEntryBase) -> Self {
        DirEntry {
            base,
            children: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, inode: u64, entry: Box<dyn FSEntry>) {
        self.children.insert(inode, entry);
    }

    pub fn get_child(&self, inode: &u64) -> Option<&Box<dyn FSEntry>> {
        self.children.get(inode)
    }

    pub fn remove_child(&mut self, inode: &u64) {
        self.children.remove(inode);
    }

    pub fn list_children(&self) -> Vec<&Box<dyn FSEntry>> {
        self.children.values().collect()
    }

    pub fn list_children_range(&self, offset: usize, length: usize) -> Vec<&Box<dyn FSEntry>> {
        self.children.values().skip(offset).take(length).collect()
    }
}