#![no_std]
#![no_main]

mod sbi;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    println!("Hello world!");

    loop {}
}

#[link_section = ".text.boot"]
#[unsafe(naked)]
#[no_mangle]
unsafe extern "C" fn boot() -> ! {
    core::arch::naked_asm!("la sp, __stack_top", "j kernel_main",);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("Panic at the kernel!");
    loop {}
}
