use storage::BlockDevice::BlockDevice;
use std::error::Error;

mod storage;

pub fn main() {
    println!("Loading storage module...");
    let mut devicetest = BlockDevice::new(1, 1024 * 1024 * 1024); //1GB device
    println!("Created BlockDevice: {:?}", devicetest);


    let test_read = devicetest.read(0, 8192);
    println!("Read: {:?}", test_read);
}