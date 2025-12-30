mod mem;
mod sbi;
mod trap;

pub mod csr;
pub use sbi::print;

use crate::log;

pub fn init() {
    log::debug!("Initializing arch");

    trap::init();
}

pub use mem::map_page;
