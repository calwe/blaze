#![no_std]
#![no_main]

fn _start() {
    loop {}
}

#[panic_handler]
fn tmp_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}