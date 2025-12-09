use std::collections::HashMap;
use crate::filesystem::FSEntry::{FSEntry, FSEntryBase, Metadata};


#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn list_children(&self, offset: usize, length: usize) -> Vec<&Box<dyn FSEntry>> {
        self.children.values().skip(offset).take(length).collect()
    }
}