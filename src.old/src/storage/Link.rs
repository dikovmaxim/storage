use serde_json;
use super::Inode::{DataId, InodeId};
use super::Utils::TableLookup;
use redis::Commands;
use super::FuseMgr::FsManagable;
use crate::manager::Kvs::Kvs;
use crate::manager::Kvs::KvsStore;
use std::error::Error;
use fuse::FileAttr;
use fuse::FileType;
use time::Timespec;

// Link Structure
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Link {
    pub id: u128, // Changed to u128
    pub target_inode: u128, // Changed to u128
}

impl TableLookup for Link {
    fn get_table_id_by_id(id: u128) -> String {
        format!("link:{}", id)
    }
}

impl FsManagable for Link {
    fn get_entry_attributes(&self, inode: u128) -> FileAttr {
        FileAttr {
            ino: inode as u64,
            size: 0,
            blocks: 0,
            atime: Timespec::new(0, 0),
            mtime: Timespec::new(0, 0),
            ctime: Timespec::new(0, 0),
            crtime: Timespec::new(0, 0),
            kind: FileType::Symlink,
            perm: 0o644,
            nlink: 1,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
        }
    }
}

impl crate::manager::Kvs::KvsStore for Link {
    fn store(&self, conn: &mut redis::Connection) -> Result<(), Box<dyn std::error::Error>> {
        let key = Self::get_table_id_by_id(self.id);
        let value = serde_json::to_string(self)?;
        conn.set(key, value).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn load(id: &str, conn: &mut redis::Connection) -> Result<Self, Box<dyn std::error::Error>> {
        let value: String = conn.get(id).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        let link: Link = serde_json::from_str(&value)?;
        Ok(link)
    }
}