use arrayvec::ArrayVec;

use crate::arch::PAGE_SIZE;
use crate::misc;

use core::ptr::NonNull;

use super::VirtualMemoryFlags;
use super::addr::VirtualAddr;

/// A `Range` corresponds to an region in the virtual memory address space.
#[derive(Clone, Copy)]
pub struct Range {
    base: VirtualAddr,
    length: usize,
    flags: VirtualMemoryFlags,
    is_used: bool,
}

/// A `RangeObject` contains an array of `Ranges`. This struct is allocated on a page.
pub struct RangeObject {
    objects: arrayvec::ArrayVec<Range, NUM_RANGE>,
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

pub enum AllocError {
    FailedToAllocatePage,
    FailedToMapPage,
}

impl RangeAllocator {
    pub fn new(base: VirtualAddr) -> Self {
        Self {
            objects: None,
            base,
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
        let addr = addr.as_mut_ptr::<RangeObject>();
        debug_assert!(addr.is_aligned());

        unsafe {
            addr.write(RangeObject {
                next: None,
                objects: ArrayVec::new(),
            });

            NonNull::new_unchecked(addr)
        }
    }
}
