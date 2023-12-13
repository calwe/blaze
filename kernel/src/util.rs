use core::arch::asm;

use log::info;

pub unsafe fn outb(port: u16, byte: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") byte);
    }
}

pub fn align_down(address: u64, align_to: u64) -> u64 {
    assert!(align_to.is_power_of_two(), "Must align to power of 2");
    address & !(align_to - 1)
}

pub fn is_aligned(address: u64, align: u64) -> bool {
    align_down(address, align) == address
}
