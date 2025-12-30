#![no_std]
#![no_main]

pub mod log;
pub mod misc;

pub mod arch;
mod boot;
mod mem;

use arch::print;

use crate::mem::VirtualAddr;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    boot::init();
    arch::init();
    mem::init();

    let virtual_addr = VirtualAddr::new(0x001000);
    let physical_addr = mem::allocate_pages(1).expect("Failed to allocate memory");
    let boot_page_directory = boot::BOOT_PAGE_DIRECTORY
        .get()
        .expect("Boot not initialized");

    let flags = mem::VirtualMemoryFlags::Writeable;

    log::info!("Hello world!");

    arch::map_page(boot_page_directory, virtual_addr, physical_addr, flags)
        .expect("Failed to map page");

    arch::flush_tlb();

    let ptr = virtual_addr.as_ptr() as *mut u8;
    unsafe {
        *ptr = 10;
        log::info!("{}", *ptr);
    }

    arch::halt();
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    use colorz::Colorize;
    print!(
        "{}{}{} {}\n",
        "[".red(),
        "panic".red().bold(),
        "]:".red(),
        info.red()
    );

    arch::halt();
}
