#![allow(unused)]

use crate::misc;
use core::mem::MaybeUninit;
use core::ops::Index;
use core::ptr::NonNull;

use super::VirtualMemoryFlags;
use super::addr::VirtualAddr;

use crate::arch::PAGE_SIZE;
use bitflags::bitflags;

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

    pub fn allocate_range(&mut self, size: usize, flags: VirtualMemoryFlags) -> VirtualAddr {
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

    pub fn iter(&self) -> RangeObjectIter<'_> {
        RangeObjectIter {
            current: 0,
            size: self.size,
            object: self,
        }
    }
}

pub struct RangeObjectIter<'a> {
    current: usize,
    size: usize,
    object: &'a RangeObject,
}

impl<'a> Iterator for RangeObjectIter<'a> {
    type Item = Range;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.size {
            return None;
        }

        Some(self.object[self.size])
    }
}

impl Index<usize> for RangeObject {
    type Output = Range;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.size);
        unsafe { self.objects[index].assume_init_ref() }
    }
}
