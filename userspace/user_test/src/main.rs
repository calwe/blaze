#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn _start() {
    loop {}
}

#[panic_handler]
fn tmp_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
