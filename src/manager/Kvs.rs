#![allow(unused_imports)]
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

    fn get_redis_connection(&self) -> MutexGuard<'_, redis::Connection> {
        self.conn.lock().unwrap()
    }

    pub fn store<T: KvsStore>(&self, item: &T) -> Result<(), Box<dyn Error>> {
        let mut conn = self.get_redis_connection();
        item.store(&mut conn)
    }

    pub fn load<T: KvsStore>(&self, id: &str) -> Result<T, Box<dyn Error>> {
        let mut conn = self.get_redis_connection();
        T::load(id, &mut conn)
    }
}

pub trait KvsStore {
    fn store(&self, conn: &mut redis::Connection) -> Result<(), Box<dyn Error>>;
    fn load(id: &str, conn: &mut redis::Connection) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}