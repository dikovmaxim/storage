use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::cmp::min;


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockDevice {
    pub id: u128,
    pub logical_size_bytes: u64,
    pub block_size_bytes: usize,
    pub generation: u32,
    pub blocks: BTreeMap<u64, Block>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub hash: [u8; 32],
    data: Vec<u8> // Placeholder for block data, will be queried from storage
}

impl Block{
    pub fn new(index: u64, hash: [u8; 32]) -> Self {
        //fill data with zeros for now
        let mut data = vec![0u8; 0];
        Block {
            index,
            hash,
            data
        }
    }

    pub fn new_block(index: u64) -> Self {
        Block {
            index,
            hash: [0u8; 32],
            data: vec![0u8; 4096], //default block size
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn write_data(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        if data.len() != self.data.len() {
            return Err("Data size does not match block size".into());
        }
        println!("Writing {} bytes to block {}", data.len(), self.index);
        Ok(())
    }

    pub fn read_data(&self, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        if length != self.data.len() {
            return Err("Requested length does not match block size".into());
        }
        //trim the data to the requested length
        Ok(vec![0u8; length])
    }


    pub fn to_string(&self) -> String {
        format!(
            "Block {{ index: {}, hash: {:x?}, actual_size_bytes: {} }}",
            self.index, self.hash, self.data.len()
        )
    }


}

impl BlockDevice {

    pub fn new(id: u128, logical_size_bytes: u64) -> Self {
        return BlockDevice {
            id,
            logical_size_bytes,
            block_size_bytes: 4096,
            generation: 1,
            blocks: BTreeMap::new(),
        }
    }

    pub fn translate_byte_to_block_index(&self, byte_offset: u64) -> Option<(u64, usize)> { //returns (block_index, offset_within_block)
        if byte_offset >= self.logical_size_bytes {
            return None;
        }
        let block_index = byte_offset / self.block_size_bytes as u64;
        let offset_within_block = (byte_offset % self.block_size_bytes as u64) as usize;
        Some((block_index, offset_within_block))
    }

    pub fn translate_span_to_block_indices(&self, byte_offset: u64, length: usize) -> Option<Vec<u64>> { //returns list of block indices
        if byte_offset >= self.logical_size_bytes {
            return None;
        }
        let mut block_indices = Vec::new();
        let mut remaining_length = length;
        let mut current_offset = byte_offset;

        while remaining_length > 0 {
            let (block_index, offset_within_block) = self.translate_byte_to_block_index(current_offset)?;
            block_indices.push(block_index);
            let space_in_block = self.block_size_bytes - offset_within_block;
            if remaining_length <= space_in_block {
                break;
            }
            remaining_length -= space_in_block;
            current_offset += space_in_block as u64;
        }

        Some(block_indices)
    }

    pub fn block_exists(&self, block_index: u64) -> bool {
        self.blocks.contains_key(&block_index)
    }

    pub fn write(&mut self, byte_offset: u64, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let block_indices = self.translate_span_to_block_indices(byte_offset, data.len())
            .ok_or("Byte offset out of bounds")?;

        let mut remaining_data = data;
        let mut current_offset = byte_offset;

        for &block_index in &block_indices {
            let (_, offset_within_block) = self.translate_byte_to_block_index(current_offset)
                .ok_or("Byte offset out of bounds")?;
            
            let space_in_block = self.block_size_bytes - offset_within_block;
            let bytes_to_write = min(remaining_data.len(), space_in_block);
            let block = self.blocks.entry(block_index).or_insert_with(|| Block::new_block(block_index)); //FIXME: calculate the hash later
            block.write_data(&remaining_data[..bytes_to_write])?;
            remaining_data = &remaining_data[bytes_to_write..];
            current_offset += bytes_to_write as u64;
            if remaining_data.is_empty() {
                break;
            }
        }
        Ok(())
    }

    pub fn read(&self, byte_offset: u64, length: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let block_indices = self.translate_span_to_block_indices(byte_offset, length)
            .ok_or("Byte offset out of bounds")?;

        let mut result = Vec::with_capacity(length);
        let mut remaining_length = length;
        let mut current_offset = byte_offset;

        for &block_index in &block_indices {

            let (_, offset_within_block) = self.translate_byte_to_block_index(current_offset)
                .ok_or("Byte offset out of bounds")?;
            
            let space_in_block = self.block_size_bytes - offset_within_block;
            let bytes_to_read = min(remaining_length, space_in_block);
            let block_data = if let Some(block) = self.blocks.get(&block_index) {
                block.read_data(self.block_size_bytes)?
            } else {
                vec![0u8; self.block_size_bytes]
            };
            result.extend_from_slice(&block_data[offset_within_block..offset_within_block + bytes_to_read]);
            remaining_length -= bytes_to_read;
            current_offset += bytes_to_read as u64;
            if remaining_length == 0 {
                break;
            }
        }
        return Ok(result)
    }




}