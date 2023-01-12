//! # Bump Allocator
//!
//! The most simple allocator design implemented for `kmalloc()` is a "bump allocator". This basic allocator simply has a `next` ptr which will point to the next availible block in the heap.
//! Each time we allocate more memory, the `next` ptr is just 'bumped' along. Freeing memory does *not* move this pointer back, as we don't keep track of what blocks are where. The only time the pointer moves backwards is if *all* of the heap is free'd.
//! From here we can just overwrite the previously allocated memory.
//! This is the only allocation method that we will use for now. Later, we will replace this with a buddy allocator.

use core::{alloc::GlobalAlloc, ptr};

use crate::util::WrappedMutex;

use super::allocator::align_up;

#[derive(Debug)]
/// A simple bump allocator
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Create an **empty** `BumpAllocator` 
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0
        }
    }

    /// Initialize the `BumpAllocator` with a heap
    pub fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for WrappedMutex<BumpAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut allocator = self.lock();

        let alloc_start = align_up(allocator.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > allocator.heap_end {
            ptr::null_mut()
        } else {
            allocator.next = alloc_end;
            allocator.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut allocator = self.lock();

        allocator.allocations -= 1;
        if allocator.allocations == 0 {
            allocator.next = allocator.heap_start;
        }
    }
}