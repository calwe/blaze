#![no_std]
#![feature(abi_x86_interrupt)]

pub mod init;
pub mod io;
pub mod structures;
pub mod util;

extern crate rlibc;

use core::panic::PanicInfo;
use yansi::Paint;

use crate::io::klog;
use crate::init::kinit;

#[no_mangle]
extern "C" fn _kmain(mbi_ptr: u64) -> ! {
    klog::init(log::LevelFilter::Trace);

    kinit();

    loop {}
}

#[panic_handler]
fn _panic(info: &PanicInfo) -> ! {
    println!("{}", Paint::on_red("KERNEL PANIC").bold());
    println!("{}", info);
    loop {}
}
