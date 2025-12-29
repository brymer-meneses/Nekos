#[cfg(target_arch = "riscv64")]
pub mod riscv64;

use crate::mem::{PhysicalAddr, VirtualAddr, VirtualMemoryFlags};
use core::arch::asm;

#[inline]
pub fn halt() -> ! {
    loop {
        unsafe {
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
        }
    }
}

#[inline]
pub fn init() {
    #[cfg(target_arch = "riscv64")]
    riscv64::init();
}

#[inline]
pub fn map_page(virtual_addr: VirtualAddr, physical_addr: PhysicalAddr, flags: VirtualMemoryFlags) {
    #[cfg(target_arch = "riscv64")]
    riscv64::map_page(virtual_addr, physical_addr, flags);
}

pub const PAGE_SIZE: u64 = 4096;

macro_rules! print {
    ($($arg:tt)*) => (
        #[cfg(target_arch = "riscv64")]
        $crate::arch::riscv64::print(format_args!($($arg)*));
    );
}

pub(crate) use print;
