mod addr;
mod heap;
mod page_allocator;
mod range_allocator;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use spin::{Mutex, Once};
use ubyte::ToByteUnit;

use crate::arch::PAGE_SIZE;
use crate::mem::heap::SlabAllocator;
use crate::mem::page_allocator::PAGE_ALLOCATOR;
use crate::mem::range_allocator::RangeAllocator;
use crate::misc::OnceLock;
use crate::{arch, boot, log, misc};

use bitflags::bitflags;
use limine::memory_map::EntryType;

pub use addr::*;

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
pub enum PageMapErr {
    UnalignedPhysicalAddr,
    UnalignedVirtualAddr,
    UnalignedSize,
    PageFrameAllocError,
}

pub trait PageDirectory {
    fn root_page_table(&self) -> PhysicalAddr;
}

pub fn init() {
    log::debug!("Setting up the paging system.");

    init_page_allocator();
    init_kernel_page_directory();
}

fn init_page_allocator() {
    let boot_info = boot::BOOT_INFO.get().unwrap();

    let mut allocator = PAGE_ALLOCATOR.lock();
    for entry in boot_info
        .memory_map_entries
        .iter()
        .filter(|entry| entry.entry_type == EntryType::USABLE)
    {
        let base = PhysicalAddr::new(entry.base);
        let pages = (entry.length / PAGE_SIZE) as usize;

        log::debug!(
            "Adding page frame to page allocator {}",
            entry.length.bytes()
        );

        allocator.deallocate(base, pages);
    }

    log::info!("Initialized page allocator!");
}

pub static KERNEL_RANGE_ALLOCATOR: OnceLock<RangeAllocator> = OnceLock::new();

#[global_allocator]
pub static KERNEL_HEAP_ALLOCATOR: OnceLock<SlabAllocator> = OnceLock::new();

fn init_kernel_page_directory() {
    let boot_info = boot::BOOT_INFO.get().unwrap();

    let kernel_image_offset = {
        let kernel_virtual_addr =
            misc::align_down_page(ptr::addr_of!(boot::KERNEL_BLOB_BEGIN) as u64);
        let kernel_physical_addr = boot_info.kernel_address;
        assert!(kernel_virtual_addr >= kernel_physical_addr.addr());

        kernel_virtual_addr - kernel_physical_addr.addr()
    };

    log::debug!(
        "Kernel Image Offset {}",
        VirtualAddr::new(kernel_image_offset)
    );

    let root_page_table = allocate_pages(1, true).expect("Failed to allocate page");

    let map_section = |name: &str,
                       section_begin: *const u8,
                       section_end: *const u8,
                       flags: VirtualMemoryFlags| {
        let section_begin = misc::align_down_page(section_begin as u64);
        let section_end = misc::align_up_page(section_end as u64);

        log::debug!(
            "Mapping {} region {}-{}",
            name,
            PhysicalAddr::new(section_begin),
            PhysicalAddr::new(section_end),
        );

        let section_size = (section_end - section_begin) as usize;

        let physical_addr = PhysicalAddr::new(section_begin - kernel_image_offset);
        let virtual_addr = VirtualAddr::new(section_begin);

        crate::arch::map_page(
            root_page_table,
            virtual_addr,
            physical_addr,
            section_size,
            flags,
        )
        .expect("Failed to map page")
    };

    map_section(
        "Kernel Code",
        ptr::addr_of!(boot::KERNEL_CODE_BEGIN),
        ptr::addr_of!(boot::KERNEL_CODE_END),
        VirtualMemoryFlags::Executable,
    );

    map_section(
        "Kernel Read Only Data",
        ptr::addr_of!(boot::KERNEL_RODATA_BEGIN),
        ptr::addr_of!(boot::KERNEL_RODATA_END),
        VirtualMemoryFlags::empty(),
    );

    map_section(
        "Kernel Data",
        ptr::addr_of!(boot::KERNEL_DATA_BEGIN),
        ptr::addr_of!(boot::KERNEL_DATA_END),
        VirtualMemoryFlags::Writeable,
    );

    let boot_info = boot::BOOT_INFO.get().unwrap();
    let mut usable_memory = 0u64;

    for entry in boot_info.memory_map_entries.iter().filter(|entry| {
        let entry_type = entry.entry_type;
        entry_type == EntryType::USABLE || entry_type == EntryType::BOOTLOADER_RECLAIMABLE
    }) {
        let physical_addr = PhysicalAddr::new(entry.base);
        let virtual_addr = physical_addr.as_virtual_by_offset(boot_info.hhdm_offset);

        if entry.entry_type == EntryType::USABLE {
            log::debug!(
                "Mapping usable memory region {} with size {}",
                physical_addr,
                entry.length.bytes()
            );
            usable_memory += entry.length;
        }

        arch::map_page(
            root_page_table,
            virtual_addr,
            physical_addr,
            entry.length as usize,
            VirtualMemoryFlags::Writeable,
        )
        .expect("Failed to map page");
    }

    use ubyte::ToByteUnit;

    log::info!("Total Usable Memory {}", usable_memory.bytes());

    arch::switch_page_table(root_page_table);

    let base_addr = misc::align_up_page(ptr::addr_of!(boot::KERNEL_BLOB_END) as u64);
    let base = VirtualAddr::new(base_addr);

    let mut range_allocator = RangeAllocator::new(base, root_page_table);
    let slab_allocator =
        SlabAllocator::new(&mut range_allocator).expect("Failed to allocate range");

    KERNEL_HEAP_ALLOCATOR.set(slab_allocator);
    KERNEL_RANGE_ALLOCATOR.set(range_allocator);
}

pub fn allocate_pages(
    num_pages: usize,
    zeroed: bool,
) -> Result<PhysicalAddr, page_allocator::AllocError> {
    let mut allocator = PAGE_ALLOCATOR.lock();
    let page = allocator.allocate(num_pages)?;
    let boot_info = boot::BOOT_INFO.get().unwrap();

    if zeroed {
        unsafe {
            core::ptr::write_bytes(
                page.as_virtual_by_offset(boot_info.hhdm_offset)
                    .as_mut_ptr::<u8>(),
                0,
                PAGE_SIZE as usize * num_pages,
            )
        }
    }

    Ok(page)
}

pub fn deallocate_pages(physical_addr: PhysicalAddr, num_pages: usize) {
    let mut allocator = PAGE_ALLOCATOR.lock();
    allocator.deallocate(physical_addr, num_pages);
}

unsafe impl GlobalAlloc for OnceLock<SlabAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() >= PAGE_SIZE as usize {
            let mut range_allocator = KERNEL_RANGE_ALLOCATOR.lock();
            let range = range_allocator.allocate(layout.size(), VirtualMemoryFlags::Writeable);
            if let Ok(range) = range {
                return range.base.as_mut_ptr::<u8>();
            }
            return core::ptr::null_mut();
        }

        let mut allocator = self.lock();
        allocator
            .allocate(layout.size())
            .unwrap_or_else(|_| core::ptr::null_mut::<u8>())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() >= PAGE_SIZE as usize {
            todo!("Deallocate from range allocator");
        }

        let mut allocator = self.lock();
        let address = VirtualAddr::new(ptr::addr_of!(ptr) as u64);
        allocator.deallocate(address);
    }
}
