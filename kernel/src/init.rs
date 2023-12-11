use log::{info, trace, error};

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
    trace!("Creating IDTR");
    let idtr = IDTR::new(&IDT);
    trace!("Loading IDTR");
    idtr.load();
    unsafe {
        let test = *(0x0000_8000_0000_0000 as *const u64);
        asm!("mov rax, rbx");
    }
    trace!("IDT Loaded! Testing...");
    trace!("Returned from interrupt");
}

#[no_mangle]
extern "x86-interrupt" fn breakpoint(_stack_frame: &InterruptStackFrame) {
    error!("Breakpoint!");
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
