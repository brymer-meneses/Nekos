#![allow(unused)]

use crate::mem::{PhysicalAddr, VirtualAddr};
use core::marker::PhantomPinned;
use core::{num, ptr::NonNull};

use nekos_arch::PAGE_SIZE;

use spin::Mutex;

pub static PAGE_ALLOCATOR: Mutex<FreeListAllocator> = Mutex::new(FreeListAllocator::new());

pub struct FreeListAllocator {
    // To make modifying the doubly-linked list easier, we use a dummy node. The head of the
    // free list is after this node.
    root: FreeListNode,

    // This shouldn't be movable because we store pointers to the `root`
    _pin: PhantomPinned,
}

pub struct FreeListNode {
    next: Option<NonNull<FreeListNode>>,
    prev: Option<NonNull<FreeListNode>>,

    base: PhysicalAddr,
    num_pages: usize,
}

/// SAFETY: The nodes will only reside in the mutex anyway.
unsafe impl Send for FreeListNode {}

pub struct AllocError;

impl FreeListAllocator {
    // This should only be constructible from this module.
    const fn new() -> Self {
        FreeListAllocator {
            root: FreeListNode {
                next: None,
                prev: None,
                base: PhysicalAddr::new(0),
                num_pages: 0,
            },

            _pin: PhantomPinned,
        }
    }

    pub fn allocate(&mut self, num_pages: usize) -> Result<PhysicalAddr, AllocError> {
        let mut current = self.head().ok_or(AllocError)?;

        while let Some(next) = unsafe { current.as_ref().next } {
            let node_ref = unsafe { current.as_mut() };

            // Shrink this region.
            if node_ref.num_pages > num_pages {
                return Ok(FreeListNode::shrink(current, num_pages));
            }

            // Remove this region from the free list
            if node_ref.num_pages == num_pages {
                FreeListNode::remove(current);
                return Ok(node_ref.base);
            }

            current = next;
        }

        Err(AllocError)
    }

    pub fn deallocate(
        &mut self,
        phys_addr: PhysicalAddr,
        num_pages: usize,
        phys_to_virt: fn(PhysicalAddr) -> VirtualAddr,
    ) {
        let mut current = NonNull::from_mut(&mut self.root);

        while let Some(next) = unsafe { current.as_ref().next } {
            if unsafe { next.as_ref().base } > phys_addr {
                break;
            }
            current = next;
        }

        let new_node = unsafe { FreeListNode::from_addr(phys_addr, num_pages, phys_to_virt) };
        FreeListNode::append(current, new_node);
        self.coalesce(new_node);
    }

    pub fn append_node(&mut self, node: NonNull<FreeListNode>) {
        match self.tail() {
            Some(tail) => unsafe {
                assert!(tail.as_ref().base < node.as_ref().base);
                FreeListNode::append(tail, node);
                self.coalesce(node);
            },
            None => {
                FreeListNode::append(NonNull::from_mut(&mut self.root), node);
            }
        }
    }

    fn coalesce(&mut self, mut node: NonNull<FreeListNode>) {
        let prev = unsafe { node.as_ref().prev };
        let next = unsafe { node.as_ref().next };

        if let Some(prev) = prev {
            if FreeListNode::is_adjacent(prev, node) {
                FreeListNode::extend(prev, node);
                node = prev;
            }
        }

        if let Some(next) = next {
            if FreeListNode::is_adjacent(node, next) {
                FreeListNode::extend(node, next);
            }
        }
    }

    const fn head(&self) -> Option<NonNull<FreeListNode>> {
        self.root.next
    }

    fn tail(&self) -> Option<NonNull<FreeListNode>> {
        let mut current = self.head()?;
        while let Some(next) = unsafe { current.as_ref().next } {
            current = next;
        }
        Some(current)
    }
}

impl FreeListNode {
    pub unsafe fn from_addr(
        phys_addr: PhysicalAddr,
        num_pages: usize,
        phys_to_virt: fn(PhysicalAddr) -> VirtualAddr,
    ) -> NonNull<FreeListNode> {
        let virt_addr = phys_to_virt(phys_addr);
        let ptr = virt_addr.as_mut_ptr() as *mut FreeListNode;
        debug_assert!(ptr.is_aligned());

        let node = FreeListNode {
            next: None,
            prev: None,
            base: phys_addr,
            num_pages,
        };

        unsafe { ptr.write(node) };
        unsafe { NonNull::new_unchecked(ptr) }
    }

    pub fn extend(mut left: NonNull<FreeListNode>, right: NonNull<FreeListNode>) {
        FreeListNode::remove(right);

        let left = unsafe { left.as_mut() };
        let right = unsafe { right.as_ref() };

        left.num_pages += right.num_pages;
    }

    pub fn is_adjacent(left: NonNull<FreeListNode>, right: NonNull<FreeListNode>) -> bool {
        let right = unsafe { right.as_ref() };
        FreeListNode::end(left) == right.base
    }

    pub fn end(node: NonNull<FreeListNode>) -> PhysicalAddr {
        let node = unsafe { node.as_ref() };
        PhysicalAddr::new(node.base.addr() + PAGE_SIZE * node.num_pages as u64)
    }

    pub fn append(mut node: NonNull<FreeListNode>, mut next: NonNull<FreeListNode>) {
        unsafe {
            let old_next = node.as_ref().next;

            node.as_mut().next = Some(next);
            next.as_mut().prev = Some(node);
            next.as_mut().next = old_next;

            if let Some(mut old_next) = old_next {
                old_next.as_mut().prev = Some(next);
            }
        }
    }

    pub fn remove(mut node: NonNull<FreeListNode>) {
        unsafe {
            let prev = node.as_ref().prev;
            let next = node.as_ref().next;

            prev.map(|mut node| node.as_mut().next = next);
            next.map(|mut node| node.as_mut().prev = prev);

            node.as_mut().next = None;
            node.as_mut().prev = None;
        }
    }

    /// Shrinks the given node at the end.
    pub fn shrink(mut node: NonNull<FreeListNode>, num_pages: usize) -> PhysicalAddr {
        let node = unsafe { node.as_mut() };
        assert!(node.num_pages > num_pages);

        node.num_pages -= num_pages;

        let addr = node.base.addr() + PAGE_SIZE * (node.num_pages as u64);
        PhysicalAddr::new(addr)
    }
}
