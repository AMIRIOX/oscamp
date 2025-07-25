//! [ArceOS](https://github.com/arceos-org/arceos) memory management module.

#![no_std]

#[macro_use]
extern crate log;
extern crate alloc;

mod aspace;
mod backend;

pub use self::aspace::AddrSpace;

use axerrno::{AxError, AxResult};
use axhal::mem::phys_to_virt;
use axhal::paging::PagingError;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use memory_addr::{va, PhysAddr, VirtAddr};
use memory_set::MappingError;

const USER_ASPACE_BASE: usize = 0x0000;
const USER_ASPACE_SIZE: usize = 0x40_0000_0000;

static KERNEL_ASPACE: LazyInit<SpinNoIrq<AddrSpace>> = LazyInit::new();

fn mapping_err_to_ax_err(err: MappingError) -> AxError {
    warn!("Mapping error: {:?}", err);
    match err {
        MappingError::InvalidParam => AxError::InvalidInput,
        MappingError::AlreadyExists => AxError::AlreadyExists,
        MappingError::BadState => AxError::BadState,
    }
}

fn paging_err_to_ax_err(err: PagingError) -> AxError {
    warn!("Paging error: {:?}", err);
    match err {
        PagingError::NoMemory => AxError::NoMemory,
        PagingError::NotAligned => AxError::InvalidInput,
        PagingError::NotMapped => AxError::NotFound,
        PagingError::AlreadyMapped => AxError::AlreadyExists,
        PagingError::MappedToHugePage => AxError::InvalidInput,
    }
}

/// Creates a new address space for user processes.
pub fn new_user_aspace() -> AxResult<AddrSpace> {
    let mut aspace = AddrSpace::new_empty(VirtAddr::from(USER_ASPACE_BASE), USER_ASPACE_SIZE)?;
    aspace.copy_mappings_from(&kernel_aspace().lock())?;
    Ok(aspace)
}

/// Creates a new address space for kernel itself.
pub fn new_kernel_aspace() -> AxResult<AddrSpace> {
    let mut aspace = AddrSpace::new_empty(
        va!(axconfig::KERNEL_ASPACE_BASE),
        axconfig::KERNEL_ASPACE_SIZE,
    )?;
    for r in axhal::mem::memory_regions() {
        info!("[{:?}-{:?}]", r.paddr, r.paddr + r.size );
        aspace.map_linear(phys_to_virt(r.paddr), r.paddr, r.size, r.flags.into())?;
    }
    Ok(aspace)
}

/// Returns the globally unique kernel address space.
pub fn kernel_aspace() -> &'static SpinNoIrq<AddrSpace> {
    &KERNEL_ASPACE
}

/// Returns the root physical address of the kernel page table.
pub fn kernel_page_table_root() -> PhysAddr {
    KERNEL_ASPACE.lock().page_table_root()
}

/// Initializes virtual memory management.
///
/// It mainly sets up the kernel virtual memory address space and recreate a
/// fine-grained kernel page table.
pub fn init_memory_management() {
    info!("Initialize virtual memory management...");

    let kernel_aspace = new_kernel_aspace().expect("failed to initialize kernel address space");
    debug!("kernel address space init OK: {:#x?}", kernel_aspace);
    KERNEL_ASPACE.init_once(SpinNoIrq::new(kernel_aspace));
    axhal::paging::set_kernel_page_table_root(kernel_page_table_root());
}

/// Initializes kernel paging for secondary CPUs.
pub fn init_memory_management_secondary() {
    unsafe { axhal::arch::write_page_table_root(kernel_page_table_root()) };
}
