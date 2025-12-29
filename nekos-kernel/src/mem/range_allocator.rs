#![allow(unused)]

use crate::misc;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

use crate::arch::PAGE_SIZE;
use bitflags::bitflags;

use crate::mem::VirtualAddr;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct RangeFlags: u8 {
        const Writeable = 1 << 0;
        const Executable = 1 << 1;
        const UserAccessible = 1 << 2;
        const MMIO = 1 << 3;
    }
}

/// A `Range` corresponds to an region in the virtual memory address space.
pub struct Range {
    base: VirtualAddr,
    length: usize,
    flags: RangeFlags,
    is_used: bool,
}

/// A `RangeObject` contains an array of `Ranges`. This struct is allocated on a page.
pub struct RangeObject {
    objects: [MaybeUninit<Range>; NUM_RANGE],
    size: usize,
    next: Option<NonNull<RangeObject>>,
}

// Calculate the number of range struct to fill the `RangeObject` to make it so that it has at most
// the size of a page.
const NUM_RANGE: usize =
    (PAGE_SIZE as usize - size_of::<NonNull<RangeObject>>() - size_of::<usize>())
        / size_of::<Range>();

misc::const_assert!(size_of::<RangeObject>() <= PAGE_SIZE as usize);

pub struct RangeAllocator {
    objects: Option<NonNull<RangeObject>>,
    base: VirtualAddr,
}

impl RangeAllocator {
    pub fn new(base: VirtualAddr) -> Self {
        Self {
            objects: None,
            base,
        }
    }

    pub fn allocate_range(&mut self, size: usize, range_flags: RangeFlags) -> VirtualAddr {
        todo!()
    }

    fn allocate_range_object(&mut self, address: VirtualAddr) -> NonNull<RangeObject> {
        match self.tail() {
            None => todo!(),
            Some(object) => todo!(),
        }
    }

    fn tail(&self) -> Option<NonNull<RangeObject>> {
        let mut cursor = self.objects?;
        while let Some(next) = unsafe { cursor.as_ref().next } {
            cursor = next;
        }
        Some(cursor)
    }
}

impl RangeObject {
    pub unsafe fn from_addr(addr: VirtualAddr) -> NonNull<RangeObject> {
        let addr = addr.as_ptr() as *mut RangeObject;

        unsafe {
            addr.write(RangeObject {
                size: 0,
                next: None,
                objects: MaybeUninit::uninit().assume_init(),
            });

            NonNull::new_unchecked(addr)
        }
    }
}
