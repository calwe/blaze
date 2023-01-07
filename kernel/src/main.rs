#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod memory;

use limine::{LimineBootInfoRequest, LimineMemmapRequest, LimineMemoryMapEntryType};
use raw_cpuid::CpuId;
use x86_64::{
    instructions,
    structures::paging::{Page, PageTable, Translate},
    VirtAddr,
};

use crate::memory::translate_addr;

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

    let phys_mem_offset = VirtAddr::new(0);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&mmap_response) };

    let virt = VirtAddr::new(0xdeadbeaf000);
    let phys = mapper.translate_addr(virt);
    trace!("{:?} -> {:?}", virt, phys);

    let page = Page::containing_address(virt);
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let phys = mapper.translate_addr(virt);
    trace!("{:?} -> {:?}", virt, phys);

    info!("Kernel finished");

    hcf();
}

fn init() {
    gdt::init();
    interrupts::init();

    info!("Everything initialized.");
}

fn cpu_info() {
    let cpuid = CpuId::new();
    if let Some(vf) = cpuid.get_vendor_info() {
        info!("   CPU Vendor: {}", vf.as_str());
    }
}

#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    fatal!("{}", info);
    hcf();
}

/// Die, spectacularly.
pub fn hcf() -> ! {
    loop {
        core::hint::spin_loop();
    }
}
