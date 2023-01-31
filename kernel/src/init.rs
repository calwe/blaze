//! The init module contains the kernel's init function and other init related functions.

use core::{arch::asm, borrow::BorrowMut};

use limine::LimineMemoryMapEntryType;
use raw_cpuid::CpuId;
use spin::Mutex;
use x86_64::{instructions::port::Port, structures::paging::OffsetPageTable, VirtAddr};

use crate::{
    acpi::{
        hpet::{global_hpet, GLOBAL_HPET},
        madt::{MADTEntryTypes, IOAPIC_0},
        rsdp::RSDPDescriptor,
        rsdt::RSDT,
    },
    gdt, info, interrupts,
    memory::{
        self,
        allocator::{self, HEAP_SIZE, HEAP_START},
        BootInfoFrameAllocator,
    },
    trace, warn, BOOTLOADER_INFO, MEMORY_MAP, MODULES, RSDP,
};

/// Global PageTable mapper used in the kernel
pub static MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
/// Global physical frame allocator, using the BootInfo from Limine
pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);

/// The root init function for the kernel.
pub fn kinit() {
    gdt::init();
    interrupts::init();
    init_memory();
    init_acpi();

    info!("Sleeping for 1s");
    global_hpet().sleep(1000);
    info!("Back!");

    info!("Sleeping for 500ms");
    global_hpet().sleep(500);
    info!("Back!");

    info!("Sleeping for 100ms");
    global_hpet().sleep(100);
    info!("Back!");

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
    // then we get the RSDT so that we can parse the ACPI tables
    let rsdp_resp = RSDP
        .get_response()
        .get()
        .expect("Bootloader did not respond to RSDP request.");
    let rsdp = RSDPDescriptor::from_rsdp_response(rsdp_resp);
    let rsdt_ptr = RSDT::from_addr(rsdp.rsdt_address());
    let rsdt = unsafe { &*rsdt_ptr };

    // MADT
    // the MADT contains infomation we need to initialise the LocalAPICs and the IOAPICs
    // first we need to disable the PIC
    disable_pic();

    let madt = unsafe { &*rsdt.get_madt().unwrap() };
    // TODO: Should this be moved into a MADT initialisation function?
    for entry in madt.entries() {
        trace!("MADT Entry of type {:x?}", entry.get_type());
        // There are multiple types of entries in the MADT, such as InterruptSourceOverrides,
        // and also IOAPICs vs LocalAPICs
        match entry.get_type() {
            Some(MADTEntryTypes::IOAPIC(ioapic)) => {
                let ioapic0 = *IOAPIC_0.lock();
                if let Some(_) = ioapic0 {
                    warn!("Multiple IOAPICs found, yet only 1 is supported. Ignored.");
                    continue;
                } else {
                    IOAPIC_0.lock().replace(ioapic);
                }

                // renable interrupts
                unsafe {
                    asm!("sti", options(nostack, nomem, preserves_flags));
                }
            }
            // TODO: Parse the other MADT entrys
            _ => {}
        }
    }

    // HPET
    // we find the HPET in the RSDT and assign this into a global
    unsafe {
        *GLOBAL_HPET.borrow_mut() = Some(Mutex::new(rsdt.get_hpet().unwrap()));
    };
    // then it can be initialsed for future use.
    global_hpet().init();
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
