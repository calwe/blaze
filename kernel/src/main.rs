//! # PrivOS Kernel
//!
//! A simple kernel for the PrivOS operating system.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(ptr_metadata)]
#![warn(missing_docs)]

extern crate alloc;

pub mod acpi;
pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod loader;
pub mod memory;
pub mod util;

use core::arch::global_asm;

use limine::{
    LimineBootInfoRequest, LimineMemmapRequest, LimineMemoryMapEntryType, LimineModuleRequest,
    LimineRsdpRequest,
};
use raw_cpuid::CpuId;
use spin::Mutex;
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

use crate::{
    acpi::{
        rsdp::{RSDPDescriptor, RSDPDescriptor20},
        rsdt::{ACPISDTHeader, RSDT},
    },
    loader::elf::{load_elf_at_addr, Elf64_Ehdr},
    memory::{
        allocator::{self, HEAP_SIZE, HEAP_START},
        translate_addr, BootInfoFrameAllocator,
    },
};

use lazy_static::lazy_static;

static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
static MEMORY_MAP: LimineMemmapRequest = LimineMemmapRequest::new(0);
static MODULES: LimineModuleRequest = LimineModuleRequest::new(0);
static RSDP: LimineRsdpRequest = LimineRsdpRequest::new(0);

global_asm!(include_str!("asm/usermode.S"));

extern "C" {
    fn _usermode_jump(func: u64, stack: u64);
}

const USER_STACK_START: u64 = 0xffff_dead_beef_0000;
const USER_STACK_SIZE: u64 = 1024 * 100; // 100KiB
const USER_FUNCTION_START: u64 = 0xffff_ffff_feef_0000;

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

    trace!("Getting modules from bootloader");
    let modules = MODULES
        .get_response()
        .get()
        .expect("Bootloader did not respond to modules request.");
    let modules = modules.modules();
    let ramdisk = modules.get(0).expect("No ramdisk at index 0");
    trace!("ramdisk size: {}", ramdisk.length);
    trace!("ramdisk base: {:p}", ramdisk.base.as_ptr().unwrap());

    let rsdp_resp = RSDP
        .get_response()
        .get()
        .expect("Bootloader did not respond to RSDP request.");
    let rsdp = RSDPDescriptor::from_rsdp_response(rsdp_resp);
    trace!("{:?}", rsdp);
    let rsdt_ptr = RSDT::from_addr(rsdp.rsdt_address());
    let rsdt = unsafe { &*rsdt_ptr };
    let madt = unsafe { &*rsdt.get_madt().unwrap() };
    trace!("MADT: {:?}", madt);
    for entry in madt.entries() {
        trace!("MADT Entry of type {:?}", entry.get_type());
    }

    info!("Kernel finished");

    //let entry = load_elf_at_addr(ramdisk.base.as_ptr().unwrap() as u64).unwrap();

    // info!("jumping to usermode: {:x}", entry.0);
    // unsafe {
    //     _usermode_jump(entry.0, entry.1);
    // }

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
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&mmap_response) };

    trace!(
        "Mapping kernel heap (0x{:x} -> 0x{:x})",
        HEAP_START,
        HEAP_START + HEAP_SIZE
    );
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("initialising heap failed");

    trace!(
        "Mapping user stack (0x{:x} -> 0x{:x})",
        USER_STACK_START,
        USER_STACK_START + USER_STACK_SIZE
    );

    let page_range = {
        let heap_start = VirtAddr::new(USER_STACK_START as u64);
        let heap_end = heap_start + USER_STACK_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator.allocate_frame().unwrap();
        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut frame_allocator)
                .unwrap()
                .flush()
        };
    }

    let user_function_curr = VirtAddr::new(_usermode_function as *const () as u64);
    let user_function_phys =
        unsafe { translate_addr(user_function_curr, phys_mem_offset).unwrap() };
    let page_phys_start = (user_function_phys.as_u64() >> 12) << 12;
    let fn_page_offset = user_function_phys.as_u64() - page_phys_start;
    let user_fn_virt_base = USER_FUNCTION_START;
    let user_fn_virt = user_fn_virt_base + fn_page_offset;

    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(user_fn_virt));
    let flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
    let frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(PhysAddr::new(page_phys_start));
    unsafe {
        mapper
            .map_to(page, frame, flags, &mut frame_allocator)
            .unwrap()
            .flush();
    }

    // trace!("jumping to usermode ({:x})", user_fn_virt);
    // unsafe { _usermode_jump(user_fn_virt); }
}

/// Prints basic cpu infomation onto the screen
fn cpu_info() {
    let cpuid = CpuId::new();
    if let Some(vf) = cpuid.get_vendor_info() {
        info!("    CPU Vendor: {}", vf.as_str());
    }
    if let Some(fi) = cpuid.get_feature_info() {
        // CPU features: listed as needed
        info!("    CPU Features: {:?}", fi);
        info!("        X2APIC: {}", fi.has_x2apic());
    }
}

// // FIXME: This is an temporary hack. THIS MUST BE DELETED AND IMPROVED
// fn usermode_jump() {
//     let func = usermode_function as *const ();
//     warn!("JUMPING INTO USERMODE (at {:p})", func);
//     unsafe {
//         asm!(
//             "mov ax, (8 * 8) | 3",
//             "xchg bx, bx",
//             "mov ds, ax",
//             "mov es, ax",
//             "mov fs, ax",
//             "mov gs, ax",
//             "mov rax, rsp",
//             "push (8 * 8) | 3",
//             "push rax",
//             "pushf",
//             "push (7 * 8) | 3",
//             "push {}",
//             "iretq",
//             sym usermode_function,
//         );
//     }
// }

// FIXME: Also temporary. This should be replaced by another binary linked to the kernel.
/// tmp usermode func
#[no_mangle]
pub extern "C" fn _usermode_function() {
    loop {}
    warn!("Usermode?");
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
