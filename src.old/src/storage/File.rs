use super::FuseMgr::FsManagable;
use serde_json;

use super::Inode::{DataId, Sha256Hash};
use super::Utils::TableLookup;
use redis::Commands;
use crate::manager::Kvs::Kvs;
use crate::manager::Kvs::KvsStore;
use std::error::Error;
use fuse::FileAttr;
use fuse::FileType;
use crate::storage::Inode::InodeId;
use crate::storage::Inode::InodeKind;
use crate::storage::Inode::Inode;
use crate::storage::Directory::Directory;
use crate::storage::FuseMgr::CopyableFs;
use crate::storage::Utils::generate_unique_id;
use time::Timespec;


// File Structure (basic representation)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct File {
    pub id: u128, // Changed to u128
    pub content_hash: Sha256Hash,
    pub size: u64,
    pub block_size: u64,
}

impl File {
    // Additional methods related to File can be added here
    pub fn get_by_id(id: u128, kvs: &mut &Kvs) -> Result<File, Box<dyn Error>> {
        let mut conn = kvs.get_redis_connection();
        <File as KvsStore>::load(&<File as TableLookup>::get_table_id_by_id(id), &mut *conn)
    }

    pub fn create_new(parent: u128, name: String, kvs: &mut &Kvs) -> Result<(Inode, File), Box<dyn Error>> {
        let file_id = generate_unique_id();
        let inode_id = generate_unique_id();

        let new_file = File {
            id: file_id,
            content_hash: [0u8; 32], // Placeholder hash
            size: 0,
            block_size: 4096, // Example block size
        };

        let new_inode = Inode {
            id: inode_id,
            target: file_id,
            kind: InodeKind::File,
        };

        //println!("Creating new file '{}' with inode id {} under parent inode {}", name, inode_id, parent);
        //println!("Created Inode: {:?} for File: {:?}", new_inode, new_file.id);

        // Store the new file and inode in KVS
        {
            let mut conn = kvs.get_redis_connection();
            new_file.store(&mut *conn)?;
            new_inode.store(&mut *conn)?;
        }

        let mut parent_inode = Inode::get_by_id(parent, kvs)?;
        if parent_inode.kind != InodeKind::Directory {
            return Err(Box::from("Parent inode is not a directory"));
        }

        let parent_dir = Directory::get_by_id(parent_inode.get_target(kvs)?.1, kvs)?; //FIXME: redundand and makes an extra KVS call, optimize later

        //create a new directory metadata to avoid mutability issues
        let mut new_parent_dir = parent_dir.copy_entry_with_new_id();
        new_parent_dir.add_entry(name, new_inode.id, kvs)?;
        new_parent_dir.store(&mut *kvs.get_redis_connection())?;

        //swap the target of the parent inode to point to the updated directory
        parent_inode.swap_target(new_parent_dir.id, InodeKind::Directory, kvs)?;

        //the old parent_dir will be garbage collected later

        Ok((new_inode, new_file))
    }
}


impl TableLookup for File {
    fn get_table_id_by_id(id: u128) -> String {
        format!("file:{}", id)
    }
}

impl FsManagable for File {
    fn get_entry_attributes(&self, inode: u128) -> FileAttr {
        FileAttr {
            ino: inode as u64,
            size: self.size,
            blocks: (self.size + self.block_size - 1) / self.block_size, // number of blocks based on block_size
            atime: Timespec::new(0, 0),
            mtime: Timespec::new(0, 0),
            ctime: Timespec::new(0, 0),
            crtime: Timespec::new(0, 0),
            kind: FileType::RegularFile,
            perm: 0o644,
            nlink: 1,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
        }
    }
}



impl crate::manager::Kvs::KvsStore for File {
    fn store(&self, conn: &mut redis::Connection) -> Result<(), Box<dyn std::error::Error>> {
        let key = Self::get_table_id_by_id(self.id);
        let value = serde_json::to_string(self)?;
        conn.set(key, value).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn load(id: &str, conn: &mut redis::Connection) -> Result<Self, Box<dyn std::error::Error>> {
        let value: String = conn.get(id).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        let file: File = serde_json::from_str(&value)?;
        Ok(file)
    }
}