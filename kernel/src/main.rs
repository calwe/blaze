#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod memory;
pub mod util;

use core::arch::asm;

use alloc::{boxed::Box, vec::Vec};
use limine::{LimineBootInfoRequest, LimineMemmapRequest, LimineMemoryMapEntryType};
use raw_cpuid::CpuId;
use x86_64::{
    instructions,
    structures::paging::{Page, PageTable, Translate},
    VirtAddr,
};

use crate::memory::{translate_addr, BootInfoFrameAllocator, allocator::{self, ALLOCATOR, HEAP_START, HEAP_SIZE}};

static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
static MEMORY_MAP: LimineMemmapRequest = LimineMemmapRequest::new(0);

/// Kernel Entry Point
///
/// `_start` is defined in the linker script as the entry point for the ELF file.
/// Unless the [`Entry Point`](limine::LimineEntryPointRequest) feature is requested,
/// the bootloader will transfer control to this function.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    info!("Kernel started.");

    let bootinfo = BOOTLOADER_INFO
        .get_response()
        .get()
        .expect("Bootloader did not respond to bootinfo request.");
    info!(
        "   Bootloader: {} v{}",
        bootinfo.name.to_str().unwrap().to_str().unwrap(),
        bootinfo.version.to_str().unwrap().to_str().unwrap(),
    );
    cpu_info();

    init();

    info!("Kernel finished");

    usermode_jump();

    hcf();
}

/// Initializes tables and resources that will be used in the rest of our kernel.
fn init() {
    gdt::init();
    interrupts::init();
    init_memory();

    info!("Everything initialized.");
}

/// Basic memory initialisation
/// 
/// - Get the memory map from the bootloader (and display basic infomation)
/// - Create a frame allocator using the memory map
/// - Map the kernel heap into memory
fn init_memory() {
    trace!("Getting memory map from bootloader");
    let mmap_response = MEMORY_MAP
        .get_response()
        .get()
        .expect("Bootloader did not respond to memory map request.");

    let mmap = mmap_response.memmap();
    let useable_mem = mmap
        .iter()
        .filter(|entry| entry.typ == LimineMemoryMapEntryType::Usable)
        .map(|entry| entry.len)
        .sum::<u64>()
        / 1024
        / 1024;
    info!("Usable Memory: {} MiB", useable_mem);

    trace!("Creating our frame allocator");
    let phys_mem_offset = VirtAddr::new(0);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&mmap_response)
    };

    trace!("Mapping kernel heap ({:x} -> {:x})", HEAP_START, HEAP_START + HEAP_SIZE);
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("initialising heap failed");
}

/// Prints basic cpu infomation onto the screen
fn cpu_info() {
    let cpuid = CpuId::new();
    if let Some(vf) = cpuid.get_vendor_info() {
        info!("   CPU Vendor: {}", vf.as_str());
    }
}

// FIXME: This is an temporary hack. THIS MUST BE DELETED AND IMPROVED
fn usermode_jump() {
    let func = usermode_function as *const ();
    warn!("JUMPING INTO USERMODE (at {:p})", func);
    unsafe { 
        asm!(
            "mov ax, (8 * 8) | 3",
            "xchg bx, bx",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov rax, rsp",
            "push (8 * 8) | 3",
            "push rax",
            "pushf",
            "push (7 * 8) | 3",
            "push {}",
            "iretq",
            sym usermode_function,
        ); 
    }
}

// FIXME: Also temporary. This should be replaced by another binary linked to the kernel.
#[no_mangle]
extern "C" fn usermode_function() {
    unsafe {
        asm!("xchg bx, bx");
    };
    hcf();
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    fatal!("{}", info);
    hcf();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

/// Die, spectacularly.
pub fn hcf() -> ! {
    loop {
        core::hint::spin_loop();
    }
}
