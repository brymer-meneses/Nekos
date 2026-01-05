#![no_std]
#![no_main]

pub mod log;
pub mod misc;

pub mod arch;
mod boot;
mod mem;

use arch::print;

extern crate alloc;

use alloc::*;

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    boot::init();
    arch::init();
    mem::init();

    log::info!("Hello world!");

    let vec = vec![1, 2, 3, 4];

    log::info!("{:?}", vec);

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
