#![no_std]
#![no_main]

mod sbi;
mod trap;

use core::{arch, ptr};

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    println!("Hello world!");

    trap::setup();

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("Panic at the kernel!");
    loop {}
}

extern "C" {
    static __stack_top: u8;

    static mut __bss_start: u8;
    static __bss_end: u8;
}

#[no_mangle]
unsafe extern "C" fn boot() -> ! {
    arch::asm!(
        "la sp, {stack_top}",
        stack_top = sym __stack_top,
    );

    // Zero the BSS section.
    let bss_start = ptr::addr_of!(__bss_start);
    let bss_end = ptr::addr_of!(__bss_end);
    let bss_size = (bss_start as u64 - bss_end as u64) as usize;
    ptr::write_bytes(ptr::addr_of_mut!(__bss_start), 0, bss_size);

    kernel_main();
}
