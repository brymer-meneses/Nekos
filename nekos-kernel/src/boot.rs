#![no_std]
#![no_main]

mod sbi;
mod trap;

#[no_mangle]
pub extern "C" fn kernel_main(_hart_id: u64, device_tree_addr: u64) -> ! {
    trap::setup();

    println!("Device tree! 0x{:08x}", device_tree_addr);
    loop {}
}

extern "C" {
    static __stack_top: u8;

    static mut __bss_start: u8;
    static __bss_end: u8;
}

fn boot(hart_id: u64, device_tree_addr: u64) -> ! {
    use core::ptr;

    // Zero the BSS section.
    let bss_start = ptr::addr_of!(__bss_start);
    let bss_end = ptr::addr_of!(__bss_end);
    let bss_size = (bss_end as usize) - (bss_start as usize);

    unsafe {
        ptr::write_bytes(ptr::addr_of_mut!(__bss_start), 0, bss_size);
    }

    kernel_main(hart_id, device_tree_addr);
}

#[no_mangle]
#[unsafe(naked)]
#[link_section = ".text.boot"]
extern "C" fn early_boot() -> ! {
    core::arch::naked_asm!(
        "la sp, {stack_top}",
        "call {boot}",
        stack_top = sym __stack_top,
        boot = sym boot,
    );
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("Panic at the kernel!");
    loop {}
}
