#![no_std]

pub mod print;
pub mod util;

extern crate rlibc;

use core::{panic::PanicInfo, arch::asm};

#[no_mangle]
extern "C" fn _kmain(mbi_ptr: u64) -> ! {
    // TODO: Serial Output
    println!("Hello world!");
    println!("Finally, printing args. The mbi_ptr is {mbi_ptr}");
    loop {}
}

#[panic_handler]
fn _panic(_: &PanicInfo) -> ! {
    loop {}
}
