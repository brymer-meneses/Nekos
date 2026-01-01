#[cfg(target_arch = "riscv64")]
pub mod riscv64;

use crate::mem::{PageMapErr, PhysicalAddr, VirtualAddr, VirtualMemoryFlags};
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
pub fn map_page(
    root_page_table: PhysicalAddr,
    virtual_addr: VirtualAddr,
    physical_addr: PhysicalAddr,
    size: usize,
    flags: VirtualMemoryFlags,
) -> Result<(), PageMapErr> {
    #[cfg(target_arch = "riscv64")]
    {
        return riscv64::map_page(root_page_table, virtual_addr, physical_addr, size, flags);
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
pub fn switch_page_table(page_table_addr: PhysicalAddr) {
    #[cfg(target_arch = "riscv64")]
    {
        use crate::arch::riscv64;
        use crate::arch::riscv64::csr::satp;

        use riscv64::csr::{CsrRead, CsrWrite};
        use riscv64::mem::PPN;

        let mut satp_reg = riscv64::csr::satp::read();
        let ppn = PPN::from_physical_addr(page_table_addr);
        satp_reg.set_ppn(ppn);

        unsafe {
            satp::write(satp_reg);
        }

        flush_tlb();
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
