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

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
    #[cfg(target_arch = "riscv64")]
    $crate::riscv64::log(format_args!($($arg)*))
    );
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
