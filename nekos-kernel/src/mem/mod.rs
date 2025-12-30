use bitflags::bitflags;
use limine::memory_map::EntryType;

mod addr;
mod page_allocator;
mod range_allocator;

use crate::{boot, log};

pub use addr::*;

use crate::arch::PAGE_SIZE;
use crate::mem::page_allocator::{FreeListNode, PAGE_ALLOCATOR};

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct VirtualMemoryFlags: u8 {
        const Writeable = 1 << 0;
        const Executable = 1 << 1;
        const UserAccessible = 1 << 2;
        const MMIO = 1 << 3;
    }
}

#[derive(Debug)]
pub struct PageMapErr;

pub trait PageDirectory {
    fn root_page_table_addr(&self) -> VirtualAddr;
    fn translate(&self, physical_addr: PhysicalAddr) -> VirtualAddr;
}

pub fn init() {
    log::debug!("Setting up the paging system.");

    let memory_map_entries = boot::MEMORY_MAP_ENTRIES
        .get()
        .expect("Boot not initialized yet");

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

    let boot_page_directory = boot::BOOT_PAGE_DIRECTORY
        .get()
        .expect("Boot not initialized yet");

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
            let node = unsafe { FreeListNode::from_addr(boot_page_directory, base, pages) };
            let mut allocator = PAGE_ALLOCATOR.lock();
            allocator.append_node(node)
        }
    }

    log::info!("Initialized page allocator!");
}

pub fn allocate_pages(num_pages: usize) -> Result<PhysicalAddr, page_allocator::AllocError> {
    let mut allocator = PAGE_ALLOCATOR.lock();
    allocator.allocate(num_pages)
}

pub fn deallocate_pages<T: PageDirectory>(
    page_directory: &T,
    physical_addr: PhysicalAddr,
    num_pages: usize,
) {
    let mut allocator = PAGE_ALLOCATOR.lock();
    allocator.deallocate(page_directory, physical_addr, num_pages);
}
