mod sbi;
mod trap;

pub mod csr;

pub use sbi::print;

pub fn init() {
    trap::init();
}
