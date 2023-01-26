//! Functions for our interrupt handlers

use core::{arch::asm, sync::atomic::AtomicU8};

use lazy_static::lazy_static;
use x86_64::{
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
    PrivilegeLevel,
};

use crate::{fatal, gdt, info, trace, warn, acpi::madt::MADT};

static TIMER_TICKS: AtomicU8 = AtomicU8::new(0);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault
            .set_handler_fn(gp_fault_handler);
        // !SAFETY: We know our stack index is only used by the double fault handler
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[0x80]
            .set_handler_fn(system_call)
            .set_privilege_level(PrivilegeLevel::Ring3);

        idt[0x30].set_handler_fn(timer_interrupt_handler);
        idt
    };
}

/// Initialize the IDT
pub fn init() {
    trace!("Initializing IDT");
    IDT.load();
    info!("IDT initialized");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    fatal!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    fatal!("EXCEPTION: PAGE FAULT");
    fatal!(
        "Accessed Address: {:?}",
        x86_64::registers::control::Cr2::read()
    );
    fatal!("Error Code: {:?}", error_code);
    fatal!("{:#?}", stack_frame);
    panic!();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT ({error_code})\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn gp_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT ({error_code})\n{:#?}",
        stack_frame
    );
}

extern "x86-interrupt" fn system_call(_: InterruptStackFrame) {
    info!("System call!");
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe { asm!("cli") }
    // if TIMER_TICKS.load(core::sync::atomic::Ordering::Relaxed) == 100 {
    //     info!("Timer interrupt");
    //     TIMER_TICKS.store(0, core::sync::atomic::Ordering::Relaxed);
    // }
    // TIMER_TICKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    warn!("Timer interrupt");

    // Send an EOI to the APIC
    MADT::write_apic_reg_HACK(0xB0, 0);
    unsafe { asm!("sti") }
}
