#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(ptr_metadata)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;


use core::{mem, str, ptr};
use std::os::arceos::modules::axhal::mem::{PhysAddr, phys_to_virt};

/// Physical address for pflash#1
#[cfg(target_arch = "riscv64")]
const PFLASH_START: usize = 0x2200_0000;
#[cfg(target_arch = "aarch64")]
const PFLASH_START: usize = 0x0400_0000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    #[cfg(any(target_arch = "riscv64", target_arch = "aarch64"))]
    {
        // Makesure that we can access pflash region.
        let va = phys_to_virt(PFLASH_START.into()).as_usize();
        let ptr = va as *const u32;
        unsafe {
            println!("Try to access dev region [{:#X}], got {:#X}", va, *ptr);
            let magic = mem::transmute::<u32, [u8; 4]>(*ptr);
            println!("Got pflash magic: {}", str::from_utf8(&magic).unwrap());
        }
    }
    #[cfg(target_arch = "x86_64")] {
        use axhal::ModuleEntry;
        let multiboot_info = axhal::boot_info();
        println!("{:?}", &multiboot_info);
        let mods_vaddr = phys_to_virt(PhysAddr::from(multiboot_info.mods_addr as usize)).as_usize();

        let mods_cnt: usize = multiboot_info.mods_count as usize;
        let modules = unsafe { 
            core::slice::from_raw_parts(mods_vaddr as *const ModuleEntry, mods_cnt)
        };
        for (i, md) in modules.iter().enumerate() {
            let st = md.mod_start as usize;
            let mut size = 0;
            let cmd_vaddr = phys_to_virt(PhysAddr::from(multiboot_info.cmdline as usize)).as_usize();
            let cmd = unsafe { core::ffi::CStr::from_ptr(cmd_vaddr as *const i8).to_str().unwrap() };
            for part in cmd.split_whitespace() {
                if let Some(val_str) = part.strip_prefix("ramdisk_size=") {
                    size = val_str.parse::<usize>().unwrap_or(0);
                }
            }
            let ed = st + size;
            println!(
                "Ramdisk {}: addr = [{:#x}, {:#x}), size = {} KB", 
                i, st, ed, size / 1024
            );
        }
    }
}
