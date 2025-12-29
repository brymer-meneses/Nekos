mod sbi;
mod trap;

pub mod csr;
pub use sbi::print;

use crate::mem::{PhysicalAddr, VirtualAddr, VirtualMemoryFlags};

pub fn init() {
    trap::init();
}

pub fn map_page(virtual_addr: VirtualAddr, physical_addr: PhysicalAddr, flags: VirtualMemoryFlags) {
}
