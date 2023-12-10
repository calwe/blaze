#![no_std]

pub mod klog;
pub mod util;

#[macro_use]
pub mod print;

extern crate rlibc;

use core::{panic::PanicInfo, arch::asm};

#[no_mangle]
extern "C" fn _kmain(mbi_ptr: u64) -> ! {
    klog::init(log::LevelFilter::Trace);

    loop {}
}

#[panic_handler]
fn _panic(_: &PanicInfo) -> ! {
    loop {}
}
