use std::fs::File as StdFile;
use std::io::Read;
use std::path::Path;
use sha2::{Sha256, Digest};
use rand::Rng;
use crate::filesystem::{FileEntry::FileEntry, Chunk::Chunk};
use crate::filesystem::FSEntry::{FSEntryBase, Metadata};
use std::time::{SystemTime, UNIX_EPOCH};
mod utils;
use crate::utils::checksum::calculate_checksum;

mod filesystem;

static CHUNK_SIZE: usize = 4096; // 4KB chunk size

fn main() {
    let file_primitive = FilePrimitive::createFromFile("image.png");
    println!("{}", file_primitive.to_string());
    for chunk in file_primitive.get_chunks() {
        println!("{}", chunk.to_string());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileChunk {
    chunk_id: u128,
    data: Vec<u8>,
    chunk_index: usize,
    chunk_checksum: [u8; 32], // Example: SHA-256 checksum
    file_id: u128,
}

impl FileChunk {
    fn to_string(&self) -> String {
        format!(
            "FileChunk {{ chunk_id: {}, idx: {}, checksum: {:x?} }}",
            self.chunk_id,
            self.chunk_index,
            self.chunk_checksum
        )
    }

    fn get_real_chunk(&self) -> Chunk {
        Chunk::new_from_data(self.file_id, self.chunk_index as u64, &self.data)
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
//primitive representation of a file including its chunks, not actual storage
struct FilePrimitive {
    id: u128,
    name: String,
    size: u64,
    chunks: Vec<FileChunk>,
    checksum: [u8; 32],
}

impl FilePrimitive {
    fn new(id: u128, name: String, size: u64, chunks: Vec<FileChunk>, checksum: [u8; 32]) -> Self {
        FilePrimitive {
            name,
            size,
            chunks,
            checksum,
            id,
        }
    }

    pub fn new_file_id() -> u128 {
        rand::random::<u128>()
    }

    fn createFromFile(path: &str) -> Self {
        let mut file = StdFile::open(Path::new(path)).expect("Failed to open file");
        let mut buffer = Vec::new();
        let file_id = FilePrimitive::new_file_id();
        file.read_to_end(&mut buffer).expect("Failed to read file");
        let file_size = buffer.len() as u64;
        let file_checksum = calculate_checksum(&buffer);
        let mut chunks = Vec::new();
        for (i, chunk) in buffer.chunks(CHUNK_SIZE).enumerate() {
            let bytes = chunk.to_vec();
            let chunk_checksum = calculate_checksum(&bytes);
            let chunk_index = i;
            let chunk_id = make_chunk_id(i as u64, &chunk_checksum, &file_checksum);
            let file_chunk = FileChunk {
                chunk_id,
                data: bytes,
                chunk_index,
                chunk_checksum,
                file_id,
            };
            chunks.push(file_chunk);
        }

        return FilePrimitive::new(
            file_id,
            String::from(Path::new(path).file_name().unwrap().to_str().unwrap()),
            file_size,
            chunks,
            file_checksum,

        )
    }

    fn make_real_chunks(&self) -> Vec<Chunk> {
        self.chunks.iter().map(|fc| fc.get_real_chunk()).collect()
    }

    fn get_file_struct(&self) -> FileEntry {
        let inode = rand::rng().random::<u64>();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let metadata = Metadata {
            created_at: now,
            modified_at: now,
            accessed_at: now,
            changed_at: now,
            permissions: 0o644,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
        };
        let base = FSEntryBase {
            inode,
            name: self.name.clone(),
            metadata,
        };
        //pub fn new_from_data(file_id: u128, idx: u64, data: &[u8])
        let chunks = self.make_real_chunks();

        FileEntry::new(base, self.size, chunks, self.checksum)
    }

    fn get_chunks(&self) -> &Vec<FileChunk> {
        &self.chunks
    }

    fn to_string(&self) -> String {
        format!(
            "FilePrimitive {{ name: {}, size: {}, chunks: {}, checksum: {:x?} }}",
            self.name,
            self.size,
            self.chunks.len(),
            self.checksum
        )
    }
}

fn make_chunk_id(idx: u64, chunk_checksum: &[u8; 32], file_checksum: &[u8; 32]) -> u128 {
    // Create a compact 64-bit chunk id from index and checksums (truncated SHA-256)
    let mut hasher = Sha256::new();
    hasher.update(idx.to_le_bytes());
    hasher.update(chunk_checksum);
    hasher.update(file_checksum);
    let out = hasher.finalize();
    u128::from_le_bytes(out[0..16].try_into().unwrap())
}

