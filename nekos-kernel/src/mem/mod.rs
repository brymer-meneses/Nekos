use crate::println;
use limine::memory_map::EntryType;
use limine::request::{HhdmRequest, MemoryMapRequest};

mod addr;

pub use addr::*;

#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

pub fn init() {
    println!("Setting up the paging system.");

    let hhdm_offset = HHDM_REQUEST
        .get_response()
        .expect("No HHDM response.")
        .offset();

    println!("HHDM Offset at {}", VirtualAddr::new(hhdm_offset));

    let memory_map_entries = MEMORY_MAP_REQUEST
        .get_response()
        .expect("No Memory Map response.")
        .entries();

    for entry in memory_map_entries.iter().filter(|entry| {
        let entry_type = entry.entry_type;
        entry_type == EntryType::USABLE
    }) {
        println!(
            "Found usable physical memory region at {} - {}",
            VirtualAddr::new(entry.base),
            VirtualAddr::new(entry.base + entry.length)
        );
    }
}
