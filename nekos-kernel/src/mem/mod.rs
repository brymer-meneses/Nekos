use limine::memory_map::EntryType;
use limine::request::{HhdmRequest, MemoryMapRequest};

mod addr;

pub use addr::*;

#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

pub fn init() {
    debug!("Setting up the paging system.");

    let hhdm_offset = HHDM_REQUEST
        .get_response()
        .expect("No HHDM response.")
        .offset();

    debug!("HHDM Offset at {}", VirtualAddr::new(hhdm_offset));

    let memory_map_entries = MEMORY_MAP_REQUEST
        .get_response()
        .expect("No Memory Map response.")
        .entries();

    for entry in memory_map_entries
        .iter()
        .filter(|entry| entry.entry_type == EntryType::USABLE)
    {
        debug!(
            "Found physical memory region at {} - {}, pages {}",
            VirtualAddr::new(entry.base),
            VirtualAddr::new(entry.base + entry.length),
            entry.length / 4096
        );
    }
}
