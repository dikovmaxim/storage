use redis::Commands;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::sync::MutexGuard;

pub struct Kvs {
    pub conn: Arc<Mutex<redis::Connection>>,
}

impl Kvs {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let client = redis::Client::open("redis://127.0.0.1/")?;
        let conn = client.get_connection()?;
        Ok(Kvs {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn init(&self) -> Result<(), Box<dyn Error>> {
        // Create empty directory
        let empty_dir = crate::storage::Directory::Directory {
            id: 1, // Root dir id
            entries: vec![],
        };
        empty_dir.store(&mut *self.conn.lock().unwrap())?;
        println!("Initialized empty root directory with id 1");

        // Create root inode pointing to it
        let root_inode = crate::storage::Inode::Inode {
            id: crate::storage::Inode::ROOT_INODE,
            target: 1,
            kind: crate::storage::Inode::InodeKind::Directory,
        };
        root_inode.store(&mut *self.conn.lock().unwrap())?;
        println!("Initialized root inode with id {}", crate::storage::Inode::ROOT_INODE);

        Ok(())
    }

    pub fn get_redis_connection(&self) -> MutexGuard<redis::Connection> {
        self.conn.lock().unwrap()
    }
}

pub trait KvsStore {
    fn store(&self, conn: &mut redis::Connection) -> Result<(), Box<dyn Error>>;
    fn load(id: &str, conn: &mut redis::Connection) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}