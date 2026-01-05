#[inline]
pub const fn align_up(addr: u64, align: u64) -> u64 {
    ((addr + align - 1) / align) * align
}

#[inline]
pub const fn align_down(addr: u64, align: u64) -> u64 {
    (addr / align) * align
}

#[inline]
pub const fn align_up_page(addr: u64) -> u64 {
    let align = crate::arch::PAGE_SIZE;
    ((addr + align - 1) / align) * align
}

#[inline]
pub const fn align_down_page(addr: u64) -> u64 {
    let align = crate::arch::PAGE_SIZE;
    (addr / align) * align
}

macro_rules! const_assert {
    ($($arg:tt)*) => {
        const _: () = {
           assert!($($arg)*);
        };
    };
}

pub(crate) use const_assert;

use core::cell::OnceCell;
use spin::{Mutex, MutexGuard};

#[derive(Debug)]
pub struct OnceLock<T>(OnceCell<Mutex<T>>);

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        Self(OnceCell::new())
    }

    pub fn set(&self, param: T) {
        self.0.get_or_init(|| Mutex::new(param));
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.0
            .get()
            .expect("Failed to lock mutex since it is not initialized")
            .lock()
    }
}

unsafe impl<T> Send for OnceLock<T> {}
unsafe impl<T> Sync for OnceLock<T> {}
