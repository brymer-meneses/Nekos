#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct VirtualAddr(u64);

impl VirtualAddr {
    #[inline]
    pub const fn new(value: u64) -> Self {
        VirtualAddr(value)
    }

    #[inline]
    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    #[inline]
    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[inline]
    pub const fn addr(&self) -> u64 {
        self.0
    }

    pub const fn is_aligned_with(&self, alignment: u64) -> bool {
        self.addr() & (alignment - 1) == 0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PhysicalAddr(u64);

impl PhysicalAddr {
    #[inline]
    pub const fn new(value: u64) -> Self {
        PhysicalAddr(value)
    }

    #[inline]
    pub const fn addr(&self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn as_virtual_by_offset(&self, offset: u64) -> VirtualAddr {
        VirtualAddr::new(self.0 + offset)
    }

    pub const fn is_aligned_with(&self, alignment: u64) -> bool {
        self.addr() & (alignment - 1) == 0
    }
}

use core::fmt;

impl fmt::Display for VirtualAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016X}", self.0)
    }
}

impl fmt::Display for PhysicalAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016X}", self.0)
    }
}
