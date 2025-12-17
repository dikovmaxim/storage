use serde_json;

use super::Inode::{DataId, InodeId, Inode};
use super::Utils::TableLookup;
use super::FuseMgr::FsManagable;
use redis::Commands;
use crate::manager::Kvs::Kvs;
use crate::manager::Kvs::KvsStore;
use std::error::Error;
use fuse::FileAttr;
use fuse::FileType;
use fuse::ReplyDirectory;
use time::Timespec;

// Directory Structure (basic representation)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Directory {
    pub id: u128, // Changed to u128
    pub entries: Vec<DirectoryEntry>,
}

impl Directory {
    pub fn get_by_id(id: u128, kvs: &mut &Kvs) -> Result<Directory, Box<dyn Error>> {
        let mut conn = kvs.get_redis_connection();
        <Directory as KvsStore>::load(&<Directory as TableLookup>::get_table_id_by_id(id), &mut *conn)
    }

    pub fn get_nlinks(&self) -> u32 {
        // Each directory has at least 2 links: '.' and '..'
        2 + self.entries.len() as u32
    }

    pub fn get_directory_reply(&self, mut reply: ReplyDirectory, mut offset: i64, kvs: &mut &Kvs) -> ReplyDirectory {
        // Add '.' entry
        if offset == 0 {
            reply.add(self.id as u64, 1, FileType::Directory, ".");
            reply.add(self.id as u64, 2, FileType::Directory, "..");
            offset += 2;
        }

        // Add entries
        for entry in &self.entries {
            let inode: Inode = Inode::get_by_id(entry.inode_id, kvs)
                .expect("Failed to get Inode by id");
            
            let file_type = match inode.kind {
                super::Inode::InodeKind::File => FileType::RegularFile,
                super::Inode::InodeKind::Directory => FileType::Directory,
                super::Inode::InodeKind::Symlink => FileType::Symlink,
            };

            reply.add(
                entry.inode_id as u64,
                offset + 1,
                file_type,
                &entry.name,
            );
            offset += 1;
        }


        return reply;
    }
}

// Directory Entry Structure
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub inode_id: u128, // Changed to u128
}

impl TableLookup for Directory {
    fn get_table_id_by_id(id: u128) -> String {
        format!("dir:{}", id)
    }
}

impl FsManagable for Directory {
    fn get_entry_attributes(&self, inode: u128) -> FileAttr {
        FileAttr {
            ino: inode as u64,
            size: 0,
            blocks: 0,
            atime: Timespec::new(0, 0),
            mtime: Timespec::new(0, 0),
            ctime: Timespec::new(0, 0),
            crtime: Timespec::new(0, 0),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: self.get_nlinks(),
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
        }
    }
}



impl crate::manager::Kvs::KvsStore for Directory {
    fn store(&self, conn: &mut redis::Connection) -> Result<(), Box<dyn std::error::Error>> {
        let key = Self::get_table_id_by_id(self.id);
        let value = serde_json::to_string(self)?;
        conn.set(key, value).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn load(id: &str, conn: &mut redis::Connection) -> Result<Self, Box<dyn std::error::Error>> {
        let value: String = conn.get(id).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        let dir: Directory = serde_json::from_str(&value)?;
        Ok(dir)
    }
}