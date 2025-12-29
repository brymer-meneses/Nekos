#![no_std]

pub mod riscv64;

use core::arch::asm;

pub fn halt() -> ! {
    loop {
        unsafe {
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
        }
    }
}
#[cfg(target_arch = "riscv64")]
pub const PAGE_SIZE: u64 = 4096;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        #[cfg(target_arch = "riscv64")]
        $crate::riscv64::print(format_args!($($arg)*))
    );
}
