//! # PrivOS Kernel
//!
//! A simple kernel for the PrivOS operating system.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(ptr_metadata)]
#![warn(missing_docs)]
#![allow(non_camel_case_types)]

extern crate alloc;

pub mod acpi;
pub mod cmdline;
pub mod gdt;
pub mod init;
pub mod interrupts;
pub mod io;
pub mod loader;
pub mod memory;
pub mod threading;
pub mod util;

use core::arch::{asm, global_asm};

use limine::{
    LimineBootInfoRequest, LimineKernelFileRequest, LimineMemmapRequest, LimineModuleRequest,
    LimineRsdpRequest,
};

use crate::{
    cmdline::get_cmdline_vars,
    threading::{KThread, Task},
};

/// Information about the bootloader
pub static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
/// Memory map provided by the bootloader
pub static MEMORY_MAP: LimineMemmapRequest = LimineMemmapRequest::new(0);
/// Modules provided by the bootloader, such as a ramdisk
pub static MODULES: LimineModuleRequest = LimineModuleRequest::new(0);
/// The kernel file provided by the bootloader - gives us the cmdline
pub static KERNEL_FILE: LimineKernelFileRequest = LimineKernelFileRequest::new(0);
/// The RSDP (Root System Description Pointer) provided by the bootloader
pub static RSDP: LimineRsdpRequest = LimineRsdpRequest::new(0);

global_asm!(include_str!("asm/usermode.S"));

extern "C" {
    fn _usermode_jump(func: u64, stack: u64);
}

/// Kernel Entry Point
///
/// `_start` is defined in the linker script as the entry point for the ELF file.
/// Unless the [`Entry Point`](limine::LimineEntryPointRequest) feature is requested,
/// the bootloader will transfer control to this function.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    get_cmdline_vars();
    info!("Kernel started.");

    init::trace_info();
    init::kinit();

    // let task = Task::new(test_task as u64);
    // let thread = KThread::new(task);
    //thread.switch();

    info!("Kernel finished");

    hcf();
}

#[no_mangle]
pub extern "C" fn test_task() -> ! {
    info!("Test task started");
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
        unsafe {
            asm!("hlt");
        }
        // core::hint::spin_loop();
    }
}
