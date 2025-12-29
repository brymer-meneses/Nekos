use bitflags::bitflags;
use limine::memory_map::EntryType;
use limine::request::{HhdmRequest, MemoryMapRequest};
use spin::Once;

mod addr;
mod page_allocator;
mod range_allocator;

use crate::log;

pub use addr::*;

use crate::arch::PAGE_SIZE;
use crate::mem::page_allocator::{FreeListNode, PAGE_ALLOCATOR};

#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

static HHDM_OFFSET: Once<u64> = Once::new();

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct VirtualMemoryFlags: u8 {
        const Writeable = 1 << 0;
        const Executable = 1 << 1;
        const UserAccessible = 1 << 2;
        const MMIO = 1 << 3;
    }
}

pub fn init() {
    log::debug!("Setting up the paging system.");

    HHDM_OFFSET.call_once(|| {
        HHDM_REQUEST
            .get_response()
            .expect("No HHDM response.")
            .offset()
    });

    let hhdm_offset = unsafe { *HHDM_OFFSET.get_unchecked() };

    log::debug!("HHDM Offset at {}", VirtualAddr::new(hhdm_offset));

    let memory_map_entries = MEMORY_MAP_REQUEST
        .get_response()
        .expect("No Memory Map response.")
        .entries();

    let is_memory_region_usable = |entry_type: EntryType| match entry_type {
        EntryType::USABLE => true,
        _ => false,
    };

    let entry_type_to_str = |entry_type: EntryType| match entry_type {
        EntryType::USABLE => "Usable",
        EntryType::RESERVED => "Reserved",
        EntryType::ACPI_RECLAIMABLE => "ACPI Reclaimable",
        EntryType::ACPI_NVS => "ACPI NVS",
        EntryType::BAD_MEMORY => "Bad Memory",
        EntryType::BOOTLOADER_RECLAIMABLE => "Bootloader Reclaimable",
        EntryType::EXECUTABLE_AND_MODULES => "Kernel and Modules",
        EntryType::FRAMEBUFFER => "Framebuffer",
        _ => unreachable!(),
    };

    for entry in memory_map_entries.iter() {
        let entry_type = entry_type_to_str(entry.entry_type);
        let base = PhysicalAddr::new(entry.base);
        let pages = (entry.length / PAGE_SIZE) as usize;

        log::debug!(
            "`{}' Memory at {} - {} with {} pages.",
            entry_type,
            PhysicalAddr::new(entry.base),
            PhysicalAddr::new(entry.base + entry.length),
            pages
        );

        if is_memory_region_usable(entry.entry_type) {
            let node = unsafe { FreeListNode::from_addr(base, pages, translate_hhdm) };
            let mut allocator = PAGE_ALLOCATOR.lock();
            allocator.append_node(node)
        }
    }
}

pub fn translate_hhdm(physical_addr: PhysicalAddr) -> VirtualAddr {
    let hhdm_offset = HHDM_OFFSET
        .get()
        .expect("Tried to get HHDM offset when `mem::init` is yet to be called.");

    VirtualAddr::new(physical_addr.addr() + hhdm_offset)
}

pub fn allocate_pages(num_pages: usize) -> Result<PhysicalAddr, page_allocator::AllocError> {
    let mut allocator = PAGE_ALLOCATOR.lock();
    allocator.allocate(num_pages)
}

pub fn deallocate_pages(physical_addr: PhysicalAddr, num_pages: usize) {
    let mut allocator = PAGE_ALLOCATOR.lock();
    allocator.deallocate(physical_addr, num_pages, translate_hhdm);
}
