use crate::manager::Kvs::Kvs;
use std::error::Error;
use rand::Rng;


pub trait TableLookup {
    fn get_table_id_by_id(id: u128) -> String;
}

pub fn generate_unique_id() -> u128 { //FIXME: collision handling
    let mut rng = rand::thread_rng();
    let unique_id: u128 = rng.r#gen();
    unique_id
}