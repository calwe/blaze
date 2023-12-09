#![no_std]


use core::{panic::PanicInfo, arch::asm};

#[no_mangle]
extern "C" fn _kmain(mbi_ptr: u64) -> ! {
    // TODO: Serial Output
    put_char(b'a');
    loop {}
}

#[inline]
fn put_char(c: u8) {
    unsafe {
        asm!("out 0xe9, al", in("al") c);
    }
}

#[panic_handler]
fn _panic(_: &PanicInfo) -> ! {
    loop {}
}
