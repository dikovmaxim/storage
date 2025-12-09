
use crate::filesystem::FSEntry::{FSEntry, FSEntryBase, Metadata};

#[derive(Debug, Clone, PartialEq, Eq)]
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