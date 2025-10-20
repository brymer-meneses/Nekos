#![no_std]
#![no_main]

mod exception;
mod sbi;

use core::{
    arch::{self, asm},
    ptr,
};

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Zero the BSS section.
    let bss_start = ptr::addr_of!(__bss_start);
    let bss_end = ptr::addr_of!(__bss_end);
    let bss_size = (bss_start as u64 - bss_end as u64) as usize;
    unsafe {
        ptr::write_bytes(ptr::addr_of_mut!(__bss_start), 0, bss_size);
    };

    println!("Hello world!");

    unsafe {
        asm!("csrw stvec, {}", in(reg) exception::handler as u64);
    }

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

#[link_section = ".text.boot"]
#[unsafe(naked)]
#[no_mangle]
unsafe extern "C" fn boot() -> ! {
    arch::naked_asm!(
        "la sp, {stack_top}",
        "j {kernel_main}",
        stack_top = sym __stack_top,
        kernel_main = sym kernel_main
    );
}
