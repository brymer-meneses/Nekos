#[cfg(target_arch = "riscv64")]
pub mod riscv64;

use crate::mem::{PageDirectory, PageMapErr, PhysicalAddr, VirtualAddr, VirtualMemoryFlags};
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
pub fn map_page<T: PageDirectory>(
    directory: &T,
    virtual_addr: VirtualAddr,
    physical_addr: PhysicalAddr,
    flags: VirtualMemoryFlags,
) -> Result<(), PageMapErr> {
    #[cfg(target_arch = "riscv64")]
    {
        return riscv64::map_page(directory, virtual_addr, physical_addr, flags);
    }
    #[cfg(not(target_arch = "riscv64"))]
    {
        compile_error!("Unsupported architecture - only riscv64 is supported");
    }
}

#[inline]
pub fn root_page_table() -> PhysicalAddr {
    #[cfg(target_arch = "riscv64")]
    {
        use crate::arch::riscv64::csr::CsrRead;
        let satp = riscv64::csr::satp::read();
        return satp.ppn().as_physical_addr();
    }
    #[cfg(not(target_arch = "riscv64"))]
    {
        compile_error!("Unsupported architecture - only riscv64 is supported");
    }
}

#[inline]
pub fn flush_tlb() {
    unsafe {
        #[cfg(target_arch = "riscv64")]
        core::arch::asm!("sfence.vma");
    }
}

pub const PAGE_SIZE: u64 = 4096;

macro_rules! print {
    ($($arg:tt)*) => (
        #[cfg(target_arch = "riscv64")]
        $crate::arch::riscv64::print(format_args!($($arg)*));
    );
}

pub(crate) use print;
