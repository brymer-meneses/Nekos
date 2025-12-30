#![no_std]
#![no_main]

pub mod log;
pub mod misc;

pub mod arch;
mod boot;
mod mem;

use arch::print;
use limine::BaseRevision;

use crate::mem::VirtualAddr;

#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());

    boot::init();
    arch::init();
    mem::init();

    let virtual_addr = VirtualAddr::new(0xDEADBEEF0000);
    let physical_addr = mem::allocate_pages(1).expect("Failed to allocate memory");
    let boot_page_directory = boot::BOOT_PAGE_DIRECTORY
        .get()
        .expect("Boot not initialized");

    let flags = mem::VirtualMemoryFlags::Writeable;

    arch::map_page(boot_page_directory, virtual_addr, physical_addr, flags)
        .expect("Failed to map page");

    log::info!("Hello world!");

    let ptr = virtual_addr.as_ptr() as *mut u8;
    unsafe {
        *ptr = 10;
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
