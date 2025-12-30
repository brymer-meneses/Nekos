use crate::mem::{PageDirectory, PhysicalAddr, VirtualAddr};
use crate::{arch, log};

use limine::memory_map::Entry;
use limine::paging::Mode;
use limine::request::{HhdmRequest, MemoryMapRequest, PagingModeRequest};

use spin::Once;

#[unsafe(link_section = ".requests")]
static PAGING_MODE_REQUEST: PagingModeRequest = PagingModeRequest::new();

#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

pub static PAGING_MODE: Once<Mode> = Once::new();
pub static HHDM_OFFSET: Once<u64> = Once::new();
pub static MEMORY_MAP_ENTRIES: Once<&[&Entry]> = Once::new();
pub static BOOT_PAGE_DIRECTORY: Once<LiminePageDirectory> = Once::new();

pub fn init() {
    PAGING_MODE.call_once(|| {
        PAGING_MODE_REQUEST
            .get_response()
            .expect("Paging mode not received.")
            .mode()
    });

    HHDM_OFFSET.call_once(|| {
        HHDM_REQUEST
            .get_response()
            .expect("No HHDM response.")
            .offset()
    });

    MEMORY_MAP_ENTRIES.call_once(|| {
        MEMORY_MAP_REQUEST
            .get_response()
            .expect("No memory map response")
            .entries()
    });

    BOOT_PAGE_DIRECTORY.call_once(|| unsafe {
        let root_page_table = arch::root_page_table();
        log::info!("Root page table at {root_page_table}");

        LiminePageDirectory {
            hhdm_offset: *HHDM_OFFSET.get_unchecked(),
            root_page_table: arch::root_page_table(),
        }
    });

    let paging_mode = unsafe { *PAGING_MODE.get_unchecked() };

    let paging_mode = match paging_mode {
        Mode::SV39 => "SV39",
        Mode::SV48 => "SV48",
        Mode::SV57 => "SV57",
        _ => unreachable!("Invalid paging mode"),
    };

    log::info!("Paging mode `{paging_mode}`.");
}

pub struct LiminePageDirectory {
    hhdm_offset: u64,
    root_page_table: PhysicalAddr,
}

impl PageDirectory for LiminePageDirectory {
    fn translate(&self, physical_addr: PhysicalAddr) -> VirtualAddr {
        VirtualAddr::new(physical_addr.addr() + self.hhdm_offset)
    }

    fn root_page_table_addr(&self) -> VirtualAddr {
        self.translate(self.root_page_table)
    }
}
