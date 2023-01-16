#![no_std]
#![no_main]

use core::arch::asm;

#[no_mangle]
pub extern "C" fn _start() {
    unsafe { asm!("int 80h") };
    loop {}
}

#[panic_handler]
fn tmp_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
