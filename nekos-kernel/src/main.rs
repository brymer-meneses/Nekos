#![no_std]
#![no_main]

#[macro_use]
mod log;

mod mem;
mod trap;

use limine::BaseRevision;
use nekos_arch::print;

#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());

    trap::init();
    mem::init();

    info!("Hello world!");

    nekos_arch::halt();
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    use colorz::Colorize;
    print!(
        "{}{}{} {}\n",
        "[".red(),
        "panic".red().bold(),
        "]:".red(),
        info.message().red(),
    );
    nekos_arch::halt();
}
