#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use core::{mem, str};
use std::os::arceos::modules::axhal::mem::{PhysAddr, phys_to_virt};
use std::os::arceos::modules::axdriver::{self, prelude::BlockDriverOps};
use axdriver_base::{BaseDriverOps, DeviceType};

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("VirtIO Block Device Test");
    println!("========================");
    let mut all_devices = axdriver::init_drivers();
    
    if all_devices.block.is_empty() {
        println!("No block devices found!");
        return;
    }
    
    println!("Found {} block device(s)", all_devices.block.len());
    
    let mut block_dev = all_devices.block.take_one().expect("No block device available");
    println!("Block device: {:?}", block_dev.device_name());
    println!("Device type: {:?}", block_dev.device_type());
    
    test_virtio_blk_operations(&mut block_dev);
}

fn test_virtio_blk_operations(block_dev: &mut axdriver::AxBlockDevice) {
    const SECTOR_SIZE: usize = 512;
    const TEST_SECTOR: u64 = 0;
    
    println!("\n1. Get virtio-blk device info");
    let num_blocks = block_dev.num_blocks();
    let block_size = block_dev.block_size();
    
    println!("Count: {} blocks", num_blocks);
    println!("Size: {} Bytes", block_size);
    println!("Full Capacity: {} Bytes", num_blocks * block_size as u64);

    let mut write_buffer = vec![0u8; block_size];
    let test_data = b"Hello VirtIO Block Device! This is a direct block I/O test.";
    let copy_len = test_data.len().min(write_buffer.len());
    write_buffer[..copy_len].copy_from_slice(&test_data[..copy_len]);
    
    let sector_marker = format!("SECTOR_{}", TEST_SECTOR);
    let marker_bytes = sector_marker.as_bytes();
    if marker_bytes.len() < block_size {
        let marker_start = block_size - marker_bytes.len();
        write_buffer[marker_start..].copy_from_slice(marker_bytes);
    }
    
    println!("\n2. Write to blocks {}", TEST_SECTOR);
    match block_dev.write_block(TEST_SECTOR, &write_buffer) {
        Ok(_) => {
            println!("Success to write {} Bytes to blocks {}", block_size, TEST_SECTOR);
        }
        Err(e) => {
            println!("Failed: {:?}", e);
            return;
        }
    }
    block_dev.flush();
   
    println!("\n3. Read data from block with id {}", TEST_SECTOR);
    let mut read_buffer = vec![0u8; block_size];
    match block_dev.read_block(TEST_SECTOR, &mut read_buffer) {
        Ok(_) => {
            println!("Success to read {} Bytes from block {}", block_size, TEST_SECTOR);
        }
        Err(e) => {
            println!("Failed: {:?}", e);
            return;
        }
    }
    
    println!("\n4. Test data");
    
    let read_test_data = &read_buffer[..copy_len];
    if read_test_data == &test_data[..copy_len] {
        println!("Success to match! (Data)");
    } else {
        println!("Dismatched data:");
        println!("Expect: {:?}", str::from_utf8(&test_data[..copy_len]).unwrap());
        println!("Read: {:?}", str::from_utf8(read_test_data).unwrap());
    }
    
    if marker_bytes.len() < block_size {
        let marker_start = block_size - marker_bytes.len();
        let read_marker = &read_buffer[marker_start..];
        if read_marker == marker_bytes {
            println!("Success to match! (Block mark)");
        } else {
            println!("Dismatched block mark:");
            println!("Expect: {:?}", str::from_utf8(marker_bytes).unwrap());
            println!("Read: {:?}", str::from_utf8(read_marker).unwrap());
        }
    }
    
    println!("\n Virtio-blk Device Test OK!");
}
