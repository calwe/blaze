#![no_std]

extern crate rlibc;

use core::{panic::PanicInfo, arch::asm};

#[no_mangle]
extern "C" fn _kmain(mbi_ptr: u64) -> ! {
    unsafe { let mbi = *(mbi_ptr as *const u64); }
    loop {}
}

#[panic_handler]
fn _panic(_: &PanicInfo) -> ! {
    loop {}
}
