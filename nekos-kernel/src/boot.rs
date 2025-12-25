#![no_std]
#![no_main]

mod mem;
mod sbi;
mod trap;

use limine::BaseRevision;

#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());

    trap::init();
    mem::init();

    nekos_arch::halt();
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic at the kernel!: {}", info.message());
    nekos_arch::halt();
}
