#![allow(dead_code)]

pub struct VirtualAddr(u64);

impl VirtualAddr {
    #[inline]
    pub const fn new(value: u64) -> Self {
        VirtualAddr(value)
    }

    #[inline]
    pub const fn as_mut_ptr(&self) -> *mut u8 {
        self.0 as *mut u8
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const u8 {
        self.0 as *const u8
    }

    #[inline]
    pub const fn raw(&self) -> u64 {
        self.0
    }
}

pub struct PhysicalAddr(u64);

impl PhysicalAddr {
    #[inline]
    pub const fn new(value: u64) -> Self {
        PhysicalAddr(value)
    }

    #[inline]
    pub const fn raw(&self) -> u64 {
        self.0
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
