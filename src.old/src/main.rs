#![allow(unused_imports)]

use std::fs::File as StdFile;
use std::io::Read;
use std::path::Path;
use sha2::{Sha256, Digest};
use rand::Rng;
use crate::filesystem::{FileEntry::FileEntry, Chunk::Chunk};
use crate::filesystem::FSEntry::{FSEntryBase, Metadata};
use crate::storage::FuseFS::initFuse;
use std::time::{SystemTime, UNIX_EPOCH};
mod utils;
use crate::utils::checksum::calculate_checksum;
use std::error::Error;

mod filesystem;
mod storage;
mod manager;

use crate::manager::Kvs::Kvs;

static CHUNK_SIZE: usize = 4096; // 4KB chunk size

fn main() -> Result<(), Box<dyn Error>> {

    let kvs = Kvs::new().expect("Failed to create KVS");
    //kvs.init().expect("Failed to initialize KVS");
    //println!("KVS initialized successfully");

    initFuse(kvs);

    
    return Ok(());
}
