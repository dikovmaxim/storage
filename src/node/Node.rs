use crate::filesystem::DirEntry::DirEntry;

struct Node {
    node_id: u64,
    root_dir: Box<DirEntry>,

}

impl Node {
    pub fn init(node_id: u64) -> Self {
        Node {
            node_id,
            root_dir: Box::new(
                DirEntry::new(
                    FSEntryBase {
                        inode: 0,
                        name: String::from("/"),
                        metadata: Metadata {
                            created_at: 0,
                            modified_at: 0,
                            accessed_at: 0,
                            changed_at: 0,
                            permissions: 0o755,
                            uid: 1000,
                            gid: 1000,
                            rdev: 0,
                            flags: 0,
                        },
                    }
                )
            ),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        // Placeholder for serialization logic
        vec![]
    }
}