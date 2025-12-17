
//Distinct Identifiers for Inodes, Files, Directories, and Links for clarity and type safety
pub type InodeId = u128; //Inode Identifier ONLY
pub type DataId = u128; //Generic Identifier for Files, Directories, Links, stored in different tables


pub type Sha256Hash = [u8; 32];
pub const ROOT_INODE: InodeId = 1;

use serde_json;
use super::Utils::TableLookup;
use redis::Commands;
use std::error::Error;
use crate::manager::Kvs::Kvs;
use fuse::FileAttr;
use crate::storage::Directory::Directory;
use crate::storage::File::File;
use crate::manager::Kvs::KvsStore;
use crate::storage::FuseMgr::FsManagable;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InodeKind {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Inode {
    pub id: InodeId,
    pub target: DataId, // Could be FileId, DirId, or LinkId based on kind pointer, though in the secondary storage
    pub kind: InodeKind,
}

impl Inode {
    pub fn get_by_id(id: InodeId, kvs: &mut &Kvs) -> Result<Inode, Box<dyn Error>> {
        let mut conn = kvs.get_redis_connection();
        <Inode as crate::manager::Kvs::KvsStore>::load(&format!("inode:{}", id), &mut *conn)
    }

    pub fn get_attribute(&self, kvs: &mut &Kvs) -> FileAttr {
        match self.kind {
            InodeKind::File => {
                File::get_by_id(self.target, kvs)
                    .expect("Failed to get File by id")
                    .get_entry_attributes(self.id)
            }
            InodeKind::Directory => {
                Directory::get_by_id(self.target, kvs)
                    .expect("Failed to get Directory by id")
                    .get_entry_attributes(self.id)
            }
            InodeKind::Symlink => {
                //throw unimplemented error
                unimplemented!("File attribute retrieval for Symlink Inode is not implemented yet.");
            }
        }
    }

    pub fn get_target(&self, kvs: &mut &Kvs) -> Result<(InodeKind, u128), Box<dyn Error>> {
        match self.kind {
            InodeKind::File => {
                let file = File::get_by_id(self.target, kvs)?;
                Ok((InodeKind::File, file.id))
            }
            InodeKind::Directory => {
                let dir = Directory::get_by_id(self.target, kvs)?;
                Ok((InodeKind::Directory, dir.id))
            }
            InodeKind::Symlink => {
                //throw unimplemented error
                unimplemented!("Target retrieval for Symlink Inode is not implemented yet.");
            }
        }
    }
}

impl TableLookup for Inode {
    fn get_table_id_by_id(id: u128) -> String {
        format!("inode:{}", id)
    }
}

impl crate::manager::Kvs::KvsStore for Inode {
    fn store(&self, conn: &mut redis::Connection) -> Result<(), Box<dyn Error>> {
        let key = Self::get_table_id_by_id(self.id);
        let value = serde_json::to_string(self)?;
        println!("Storing Inode with key: {}", key);
        conn.set(key, value).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn load(id: &str, conn: &mut redis::Connection) -> Result<Self, Box<dyn Error>> {
        let value: String = conn.get(id).map_err(|e| Box::new(e) as Box<dyn Error>)?;
        let inode: Inode = serde_json::from_str(&value)?;
        println!("Loaded Inode with id: {}", id);
        Ok(inode)
    }
}


// Remark: sthis strange implementation with inodes in between seems to be canonical in many file systems,
// especially distributed ones, alowwing (i guess) hot-swapping atomic updates of files and directories without changing their identity and further cleaning up dangling references.

