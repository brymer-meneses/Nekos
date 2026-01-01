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
