use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::error::Error;
use redis;

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

    //FIXME: Move from manual commands

    pub fn store<T: KvsStorable + Serialize>(&self, item: &T) -> Result<(), Box<dyn Error>> {
        let mut guard = self.conn.lock().unwrap();
        let conn = &mut *guard;
        let serialized = serde_json::to_string(item)?;
        let key = item.get_kvs_id();
        redis::cmd("SET").arg(key).arg(serialized).query::<()> (conn)?;
        Ok(())
    }

    pub fn load<T: KvsStorable + for<'de> Deserialize<'de>>(&self, id: &str) -> Result<T, Box<dyn Error>> {
        let mut guard = self.conn.lock().unwrap();
        let conn = &mut *guard;
        let serialized: String = redis::cmd("GET").arg(id).query(conn)?;
        let item: T = serde_json::from_str(&serialized)?;
        Ok(item)
    }
}


pub trait KvsStorable {
    fn store(&self, kvs: &Kvs) -> Result<(), Box<dyn Error>>;
    fn load(id: &str, kvs: &Kvs) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    fn get_kvs_id(&self) -> String;
}