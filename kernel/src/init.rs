//! The init module contains the kernel's init function and other init related functions.

use limine::LimineMemoryMapEntryType;
use raw_cpuid::CpuId;
use x86_64::VirtAddr;

use crate::{
    acpi::{rsdp::RSDPDescriptor, rsdt::RSDT},
    gdt, info, interrupts,
    memory::{
        self,
        allocator::{self, HEAP_SIZE, HEAP_START},
        BootInfoFrameAllocator,
    },
    trace, BOOTLOADER_INFO, MEMORY_MAP, MODULES, RSDP,
};

/// The root init function for the kernel.
pub fn kinit() {
    gdt::init();
    interrupts::init();
    init_memory();
    init_acpi();

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
}

fn init_acpi() {
    let rsdp_resp = RSDP
        .get_response()
        .get()
        .expect("Bootloader did not respond to RSDP request.");
    let rsdp = RSDPDescriptor::from_rsdp_response(rsdp_resp);
    let rsdt_ptr = RSDT::from_addr(rsdp.rsdt_address());
    let rsdt = unsafe { &*rsdt_ptr };
    let madt = unsafe { &*rsdt.get_madt().unwrap() };
    for entry in madt.entries() {
        trace!("MADT Entry of type {:?}", entry.get_type());
    }
    // TODO: Use the MADT for setting up the APIC (used for SMP)
}

#[allow(dead_code)]
fn get_modules() {
    // TODO: The module here is for our userspace elf. Userspace is disabled for now.
    trace!("Getting modules from bootloader");
    let modules = MODULES
        .get_response()
        .get()
        .expect("Bootloader did not respond to modules request.");
    let modules = modules.modules();
    let ramdisk = modules.get(0).expect("No ramdisk at index 0");
    trace!("ramdisk size: {}", ramdisk.length);
    trace!("ramdisk base: {:p}", ramdisk.base.as_ptr().unwrap());
}

/// Prints basic infomation about the machine
pub fn trace_info() {
    let bootinfo = BOOTLOADER_INFO
        .get_response()
        .get()
        .expect("Bootloader did not respond to bootinfo request.");
    info!(
        "    Bootloader: {} v{}",
        bootinfo.name.to_str().unwrap().to_str().unwrap(),
        bootinfo.version.to_str().unwrap().to_str().unwrap(),
    );
    cpu_info();
}

/// Prints basic cpu infomation onto the screen
fn cpu_info() {
    let cpuid = CpuId::new();
    if let Some(vf) = cpuid.get_vendor_info() {
        info!("    CPU Vendor: {}", vf.as_str());
    }
    if let Some(fi) = cpuid.get_feature_info() {
        // CPU features: listed as needed
        info!("    CPU Features:");
        info!("        X2APIC: {}", fi.has_x2apic());
    }
}
