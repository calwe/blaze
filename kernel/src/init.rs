use log::{warn, info, trace, error};

use spin::Lazy;

use crate::structures::idt::{InterruptDescriptor, InterruptStackFrame, IDT, IDTR, PageFaultErrorCode};
use crate::println;
use core::arch::asm;

static IDT: Lazy<IDT> = Lazy::new(|| {
    let mut idt = IDT::default();
    idt.breakpoint = InterruptDescriptor::new_interrupt(breakpoint);
    idt.page_fault = InterruptDescriptor::new_page_fault(page_fault);
    idt.general_protection_fault = InterruptDescriptor::new_interrupt_error_code(general_protection);
    idt
});

#[no_mangle]
pub fn kinit() {
    info!("Welcome to Blaze. Running kinit...");
    info!("With UTF-8 Support! ⎷ⰹ⊷");
    trace!("Creating IDTR");
    let idtr = IDTR::new(&IDT);
    trace!("Loading IDTR");
    idtr.load();
    trace!("IDT Loaded! Testing...");
    unsafe {
        asm!("int3");
    }
    trace!("Returned from interrupt");
}

#[no_mangle]
extern "x86-interrupt" fn breakpoint(_stack_frame: &InterruptStackFrame) {
    warn!("Recieved breakpoint interrupt");
}

extern "x86-interrupt" fn page_fault(_stack_frame: &InterruptStackFrame, page_fault: PageFaultErrorCode) {
    error!("Page fault!");
    let mut addr: u64 = 0;
    unsafe {
        asm!("mov {}, cr2", out(reg) addr);
    }
    error!("Occured @ 0x{:x}", addr);
    println!("{page_fault:x?}");
}

extern "x86-interrupt" fn general_protection(_stack_frame: &InterruptStackFrame, error_code: u64) {
    error!("General Protection Fault in Segment 0x{:x}", error_code);
}
