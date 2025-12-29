#![no_std]
#![no_main]

pub mod log;
pub mod misc;

pub mod arch;
mod mem;

use arch::print;
use limine::BaseRevision;

#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());

    arch::init();
    mem::init();

    log::info!("Hello world!");

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
        info.message().red(),
    );

    arch::halt();
}
