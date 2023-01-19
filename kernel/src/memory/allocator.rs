//! Common functions for kernel allocators

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use crate::{trace, util::WrappedMutex};

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
    allocate_of_size(
        mapper,
        frame_allocator,
        HEAP_START as u64,
        HEAP_SIZE as u64,
        false,
    )?;

    ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);

    Ok(())
}

/// Allocate a block of memory at a certain address
pub fn allocate_of_size(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    start: u64,
    size: u64,
    user: bool,
) -> Result<(), MapToError<Size4KiB>> {
    trace!("Allocating memory block of size:");
    trace!("    Start: 0x{start:x}");
    trace!("    Size: {size}B");
    trace!("    User: {user}");

    let page_range = {
        let start = VirtAddr::new(start);
        let end = start + size - 1u64;
        let start_page = Page::containing_address(start);
        let end_page = Page::containing_address(end);
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let mut flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        if user {
            flags |= PageTableFlags::USER_ACCESSIBLE;
        }
        // FIXME: This is a really bad hack to get around the fact that the first 4GiB of memory is mapped.
        //        We should really just unmap the first 4GiB of memory and then map it again ourselves.
        let _ = mapper.unmap(page);
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() }
    }

    Ok(())
}

/// Align an address down to the nearest multiple of `align`
/// Align **must** be a power of two
pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
