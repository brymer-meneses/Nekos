#[cfg(target_arch = "riscv64")]
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

pub fn init() {
    #[cfg(target_arch = "riscv64")]
    riscv64::init();
}

pub const PAGE_SIZE: u64 = 4096;

macro_rules! print {
    ($($arg:tt)*) => (
        #[cfg(target_arch = "riscv64")]
        $crate::arch::riscv64::print(format_args!($($arg)*));
    );
}

pub(crate) use print;
