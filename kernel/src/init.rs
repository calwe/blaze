//! The init module contains the kernel's init function and other init related functions.

use core::arch::asm;

use limine::LimineMemoryMapEntryType;
use raw_cpuid::CpuId;
use spin::Mutex;
use x86_64::{instructions::port::Port, structures::paging::OffsetPageTable, VirtAddr};

use crate::{
    acpi::{
        madt::{MADTEntryTypes, IOREDTBL},
        rsdp::RSDPDescriptor,
        rsdt::RSDT,
    },
    gdt, info, interrupts,
    memory::{
        self,
        allocator::{self, allocate_of_size, map_phys_to_virt, HEAP_SIZE, HEAP_START},
        BootInfoFrameAllocator,
    },
    trace,
    util::trace_mem_16,
    BOOTLOADER_INFO, MEMORY_MAP, MODULES, RSDP,
};

pub static MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);

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
    let mapper = unsafe { memory::init(phys_mem_offset) };
    MAPPER.lock().replace(mapper);
    let frame_allocator = unsafe { BootInfoFrameAllocator::init(&mmap_response) };
    FRAME_ALLOCATOR.lock().replace(frame_allocator);

    trace!(
        "Mapping kernel heap (0x{:x} -> 0x{:x})",
        HEAP_START,
        HEAP_START + HEAP_SIZE
    );
    allocator::init_heap().expect("initialising heap failed");
}

fn init_acpi() {
    // first we need to disable the PIC
    disable_pic();

    let rsdp_resp = RSDP
        .get_response()
        .get()
        .expect("Bootloader did not respond to RSDP request.");
    let rsdp = RSDPDescriptor::from_rsdp_response(rsdp_resp);
    let rsdt_ptr = RSDT::from_addr(rsdp.rsdt_address());
    let rsdt = unsafe { &*rsdt_ptr };
    let madt = unsafe { &*rsdt.get_madt().unwrap() };

    trace!("Local APIC Addr: {:#x}", madt.local_apic_address());
    let spi = madt.read_apic_reg(0xf0);
    trace!("SPI: {:#x}", spi);

    for entry in madt.entries() {
        trace!("MADT Entry of type {:x?}", entry.get_type());
        match entry.get_type() {
            Some(MADTEntryTypes::IOAPIC(ioapic)) => {
                // map the ioapic into memory at addr 0xffff_aaaa_0000_0000;
                // FIXME: do we need to find a better way to find mapping addresses?
                map_phys_to_virt(ioapic.ioapic_address as u64, 0xffff_aaaa_0000_0000, false)
                    .unwrap();

                // TODO: registers enum
                let maxirqs = (ioapic.read(1) >> 16) + 1;
                trace!("IOAPIC max irqs: {}", maxirqs);

                let mut test_entry = IOREDTBL(0);
                test_entry.set_vector(0x30);

                ioapic.write_table_entry(2, test_entry);
                trace!(
                    "IOAPIC entry 0: {:x?}",
                    IOREDTBL(ioapic.read_table_entry(2))
                );

                trace!("Enabling PIT");
                let mut port = Port::new(0x43);
                let value = 0x36u8;
                unsafe {
                    port.write(value);
                }

                trace!("Starting PIT");
                let mut port = Port::new(0x40);
                let count = 0x1234;
                unsafe {
                    port.write(count as u8 & 0xFF as u8);
                    port.write(((count & 0x00FF) >> 8) as u8);
                }
                trace!("PIT started");

                // renable interrupts
                unsafe {
                    asm!("sti", options(nostack, nomem, preserves_flags));
                }

                trace!("Interrupts enabled");
            }
            _ => {}
        }
    }
    // TODO: Use the MADT for setting up the APIC (used for SMP)
}

fn disable_pic() {
    trace!("Disabling PIC");

    unsafe {
        asm!("cli", options(nostack, nomem, preserves_flags));
    }

    mask_pic();

    // TODO: rust abstraction
    unsafe {
        asm!(
            "mov al, 0xff",
            "out 0xa1, al",
            "out 0x21, al",
            options(nostack, nomem, preserves_flags)
        );
    }
}

fn mask_pic() {
    trace!("Masking all entries in the PIC");

    let mut pic1 = Port::new(0x21);
    let mut pic2 = Port::new(0xa1);
    unsafe {
        pic1.write(0xffu8);
        pic2.write(0xffu8);
    }
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
        info!("        Family: {:#x}", fi.family_id());
        info!("        Extended Model: {}", fi.extended_model_id());
        info!("        Model: {:#x}", fi.model_id());
        info!("        Stepping: {:#x}", fi.stepping_id());
    }
}
