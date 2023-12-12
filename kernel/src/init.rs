use log::{warn, info, trace, error};

use spin::Lazy;

use crate::structures::gdt::{GDT, GDTR};
use crate::structures::idt::{InterruptDescriptor, InterruptStackFrame, IDT, IDTR, PageFaultErrorCode};
use crate::println;
use crate::structures::tss::TSS;
use core::arch::asm;
use core::default;

const IST_STACK_SIZE: usize = 1024 * 1024; // 1 MiB

static IDT: Lazy<IDT> = Lazy::new(|| {
    trace!("Creating IDT");
    let mut idt = IDT::default();
    //idt.breakpoint = InterruptDescriptor::new_interrupt(breakpoint);
    idt.double_fault = InterruptDescriptor::new_interrupt_error_code(double_fault).with_ist(1);
    idt.page_fault = InterruptDescriptor::new_page_fault(page_fault);
    idt.general_protection_fault = InterruptDescriptor::new_interrupt_error_code(general_protection);
    idt
});

static GDT: Lazy<GDT> = Lazy::new(|| {
    trace!("Creating GDT");
    GDT::with_tss(&TSS)
});

static IST1_STACK: [u8; IST_STACK_SIZE] = [0; IST_STACK_SIZE];

static TSS: Lazy<TSS> = Lazy::new(|| {
    trace!("Creating TSS");
    let mut tss = TSS::default();
    tss.ist[0] = (&IST1_STACK as *const u8) as u64;
    tss
});

#[no_mangle]
pub fn kinit() {
    info!("Welcome to Blaze. Running kinit...");
    info!("With UTF-8 Support! ⎷ⰹ⊷");
    trace!("Creating IDTR");
    let idtr = IDTR::new(&IDT);
    trace!("Loading IDTR");
    idtr.load();
    trace!("IDTR Loaded!");
    trace!("Creating GDTR");
    let gdtr = GDTR::new(&GDT);
    trace!("Loading GDTR");
    gdtr.load();
    trace!("GDTR Loaded!");
    trace!("Testing double fault");
    unsafe {
        asm!("int3");
    }
    trace!("Returned");
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

extern "x86-interrupt" fn double_fault(_stack_frame: &InterruptStackFrame, _error_code: u64) {
    error!("Double fault...");
}
