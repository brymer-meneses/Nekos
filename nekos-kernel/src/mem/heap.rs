use core::mem::MaybeUninit;

use crate::{
    arch::PAGE_SIZE,
    log,
    mem::{VirtualAddr, VirtualMemoryFlags, range_allocator::RangeAllocator},
    misc,
};

pub struct Slab {
    base: VirtualAddr,
    current: VirtualAddr,
    freelist_head: *mut u8,
    alloc_size: usize,
    size: usize,
}

unsafe impl Sync for Slab {}
unsafe impl Send for Slab {}

impl Slab {
    fn new(base: VirtualAddr, alloc_size: usize, size: usize) -> Slab {
        Slab {
            base,
            current: base,
            freelist_head: core::ptr::null_mut(),
            alloc_size,
            size,
        }
    }

    fn within(&self, address: VirtualAddr) -> bool {
        self.base <= address && address < self.base.offset_by(self.size as u64)
    }

    fn dealloc(&mut self, address: VirtualAddr) {
        let ptr = address.as_mut_ptr::<u8>();
        unsafe {
            *ptr.cast::<*mut u8>() = self.freelist_head;
        }
        self.freelist_head = ptr;
    }

    fn alloc(&mut self) -> Result<*mut u8, AllocError> {
        if !self.freelist_head.is_null() {
            let next = unsafe { *self.freelist_head.cast::<*mut u8>() };
            let current = self.freelist_head;
            self.freelist_head = next;
            return Ok(current);
        }

        if self.current >= self.base.offset_by(self.size as u64) {
            return Err(AllocError::FullCapacity);
        }

        let alloc = self.current;
        self.current = self.current.offset_by(self.alloc_size as u64);

        Ok(alloc.as_mut_ptr())
    }
}

pub struct SlabAllocator {
    /// 8 -> 2048
    slabs: [Slab; 8],
}

#[derive(Debug)]
pub enum AllocError {
    RangeAllocErr,
    FullCapacity,
    OutOfRange,
}

impl SlabAllocator {
    pub fn new(range_allocator: &mut RangeAllocator) -> Result<Self, AllocError> {
        let mut slabs = [const { MaybeUninit::uninit() }; 8];

        for i in 0..8usize {
            let alloc_size = 2usize.pow(i as u32 + 3);
            let size = 3 * PAGE_SIZE;
            let range = range_allocator
                .allocate(size as usize, VirtualMemoryFlags::Writeable)
                .expect("failed to alloc range");

            slabs[i].write(Slab::new(range.base, alloc_size, size as usize));
        }

        Ok(SlabAllocator {
            slabs: unsafe { core::mem::transmute::<_, [Slab; 8]>(slabs) },
        })
    }

    pub fn allocate(&mut self, size: usize) -> Result<*mut u8, AllocError> {
        for slab in self.slabs.iter_mut() {
            if slab.alloc_size >= size {
                return slab.alloc();
            }
        }

        Err(AllocError::OutOfRange)
    }

    pub fn deallocate(&mut self, address: VirtualAddr) {
        for slab in self.slabs.iter_mut() {
            if slab.within(address) {
                slab.dealloc(address);
                return;
            }
        }
    }
}
