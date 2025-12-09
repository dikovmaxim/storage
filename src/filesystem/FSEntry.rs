pub trait FSEntry {
    fn get_name(&self) -> &str;
    fn get_metadata(&self) -> &Metadata;
    fn get_inode(&self) -> u64;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FSEntryBase {
    pub inode: u64,
    pub name: String,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    created_at: u64,
    modified_at: u64,
    permissions: u16,
}