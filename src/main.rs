use storage::BlockDevice::BlockDevice;
pub use ::core::fmt;

mod manager;
mod storage;

pub fn main() {
    println!("Loading storage module...");
    let mut devicetest = BlockDevice::new(1, 1024 * 1024 * 1024); //1GB device
    println!("Created BlockDevice: {:?}", devicetest);


    //let test_read = devicetest.read(0, 1024).unwrap();
    //println!("Read: {:?}", test_read);

    // Write some data
    let write_data = vec![6u8; 10];
    devicetest.write(40, &write_data).unwrap();

    let test_read_after_write = devicetest.read(0, 100).unwrap();
    let hex_output: Vec<String> = test_read_after_write.iter().map(|b| format!("{:02x}", b)).collect();
    println!("Read after write: {:?}", hex_output);
}
