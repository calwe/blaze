//! Common functions for kernel allocators

use x86_64::{structures::paging::{Size4KiB, FrameAllocator, mapper::MapToError, Page, PageTableFlags, Mapper}, VirtAddr};

use crate::{util::WrappedMutex};

use super::bump_alloc::BumpAllocator;

/// Memory address of the start of the heap
pub const HEAP_START: usize = 0xffff_ffff_dead_0000;
/// Size of the heap in bytes
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
/// Global allocator for the kernel
pub static ALLOCATOR: WrappedMutex<BumpAllocator> = WrappedMutex::new(BumpAllocator::new());

/// Map the kernel heap in memory
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);

    Ok(())
}

/// Align an address down to the nearest multiple of `align`
/// Align **must** be a power of two
pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}