use arrayvec::ArrayVec;

use crate::arch::PAGE_SIZE;
use crate::mem::PhysicalAddr;
use crate::misc;

use core::ptr::NonNull;

use super::VirtualMemoryFlags;
use super::addr::VirtualAddr;

/// A `Range` corresponds to an region in the virtual memory address space.
#[derive(Clone, Copy)]
pub struct Range {
    pub base: VirtualAddr,
    pub length: usize,
    pub flags: VirtualMemoryFlags,
    pub is_used: bool,
}

/// A `RangeObject` contains an array of `Ranges`. This struct is allocated on a page.
pub struct RangeObject {
    ranges: arrayvec::ArrayVec<Range, NUM_RANGE>,
    next: Option<NonNull<RangeObject>>,
}

// Calculate the number of range struct to fill the `RangeObject` to make it so that it has at most
// the size of a page.
const NUM_RANGE: usize =
    (PAGE_SIZE as usize - size_of::<NonNull<RangeObject>>() - size_of::<usize>())
        / size_of::<Range>();

misc::const_assert!(size_of::<RangeObject>() <= PAGE_SIZE as usize);

unsafe impl Sync for RangeAllocator {}
unsafe impl Send for RangeAllocator {}

pub struct RangeAllocator {
    objects: Option<NonNull<RangeObject>>,
    base: VirtualAddr,
    root_page_table: PhysicalAddr,
}

pub enum AllocError {
    FailedToAllocatePage,
    FailedToMapPage,
}

impl RangeAllocator {
    pub fn new(base: VirtualAddr, root_page_table: PhysicalAddr) -> Self {
        Self {
            objects: None,
            root_page_table,
            base,
        }
    }

    pub fn allocate(
        &mut self,
        length: usize,
        flags: VirtualMemoryFlags,
    ) -> Result<&mut Range, AllocError> {
        let range_object = unsafe { self.get_or_allocate_range_object()?.as_mut() };

        let adjusted_length = misc::align_up(length as u64, PAGE_SIZE);

        let i = range_object.ranges.len();
        let range = Range {
            base: self.base,
            flags,
            is_used: true,
            length: adjusted_length as usize,
        };

        range_object.ranges.push(range);
        self.base = VirtualAddr::new(self.base.addr() + adjusted_length);

        Ok(&mut range_object.ranges[i])
    }

    pub fn find_mut(&mut self, addr: VirtualAddr) -> Option<&mut Range> {
        let mut cursor = self.objects;

        while let Some(mut object) = cursor {
            let object = unsafe { object.as_mut() };

            let range = object.ranges.iter_mut().find(|range| range.is_within(addr));
            if range.is_some() {
                return range;
            }

            cursor = object.next;
        }

        None
    }

    pub fn find(&self, addr: VirtualAddr) -> Option<&Range> {
        let mut cursor = self.objects;

        while let Some(object) = cursor {
            let object = unsafe { object.as_ref() };

            let range = object.ranges.iter().find(|range| range.is_within(addr));
            if range.is_some() {
                return range;
            }

            cursor = object.next;
        }

        None
    }

    fn get_or_allocate_range_object(&mut self) -> Result<NonNull<RangeObject>, AllocError> {
        let mut allocate_range_object = || -> Result<NonNull<RangeObject>, AllocError> {
            let base = self.base;
            self.base = VirtualAddr::new(PAGE_SIZE + self.base.addr());

            let page = crate::mem::allocate_pages(1, /*zerod=*/ true)
                .map_err(|_| AllocError::FailedToAllocatePage)?;

            crate::arch::map_page(
                self.root_page_table,
                base,
                page,
                PAGE_SIZE as usize,
                VirtualMemoryFlags::Writeable,
            )
            .map_err(|_| AllocError::FailedToMapPage)?;

            assert_eq!(
                crate::arch::root_page_table(),
                self.root_page_table,
                "`self.root_page_table` is not active."
            );

            unsafe { Ok(RangeObject::from_addr(base)) }
        };

        match self.objects {
            None => {
                let range_object = allocate_range_object()?;
                self.objects = Some(range_object);

                Ok(range_object)
            }

            Some(mut range_object) => {
                let range_object_ref = unsafe { range_object.as_mut() };
                if !range_object_ref.ranges.is_full() {
                    return Ok(range_object);
                }

                let new_range_object = allocate_range_object()?;
                range_object_ref.next = Some(new_range_object);
                return Ok(new_range_object);
            }
        }
    }
}

impl RangeObject {
    pub unsafe fn from_addr(addr: VirtualAddr) -> NonNull<RangeObject> {
        let addr = addr.as_mut_ptr::<RangeObject>();
        debug_assert!(addr.is_aligned());

        unsafe {
            addr.write(RangeObject {
                next: None,
                ranges: ArrayVec::new(),
            });

            NonNull::new_unchecked(addr)
        }
    }
}

impl Range {
    pub fn is_within(&self, addr: VirtualAddr) -> bool {
        self.base <= addr && addr < self.base.offset_by(self.length as u64)
    }
}
