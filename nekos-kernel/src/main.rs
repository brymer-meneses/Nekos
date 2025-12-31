#![no_std]
#![no_main]

pub mod log;
pub mod misc;

pub mod arch;
mod boot;
mod mem;

use arch::print;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    boot::init();
    arch::init();
    mem::init();

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
