use crate::arch::PAGE_SIZE;
use crate::mem::{KERNEL_RANGE_ALLOCATOR, VirtualAddr, VirtualMemoryFlags};

pub fn handle_page_fault(faulting_addr: VirtualAddr) {
    let range_allocator = KERNEL_RANGE_ALLOCATOR.lock();
    let range = range_allocator.find(faulting_addr);
    match range {
        None => panic!("Invalid memory access {faulting_addr}"),
        Some(range) => {
            if !range.flags.contains(VirtualMemoryFlags::Writeable) {
                panic!("Invalid memory access {faulting_addr}")
            }

            let page =
                crate::mem::allocate_pages(1, /*zerod*/ true).expect("Failed to allocate page");
            crate::arch::map_page(
                range_allocator.root_page_table,
                faulting_addr,
                page,
                PAGE_SIZE as usize,
                VirtualMemoryFlags::Writeable,
            )
            .expect("Failed to map page")
        }
    }
}
