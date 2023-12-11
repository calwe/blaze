use bitfield::bitfield;
use core::arch::asm;

#[repr(C, packed)]
pub struct IDTR {
    size: u16,
    offset: u64,
}

impl IDTR {
    pub fn new(idt: &IDT) -> Self {
        Self {
            size: (256 * 16) - 1,
            offset: (idt as *const IDT) as u64
        }
    }

    pub fn load(&self) {
        unsafe {
            asm!("lidt [{}]", in(reg) self)
        }
    }
}

#[repr(C, packed)]
pub struct IDT {
    pub division_error: InterruptDescriptor,
    pub debug: InterruptDescriptor,
    pub nmi: InterruptDescriptor,
    pub breakpoint: InterruptDescriptor,
    pub overflow: InterruptDescriptor,
    pub bound_range_exceeded: InterruptDescriptor,
    pub invalid_opcode: InterruptDescriptor,
    pub device_not_available: InterruptDescriptor,
    pub double_fault: InterruptDescriptor,
    pub coprocessor_segment_overrun: InterruptDescriptor,
    pub invalid_tss: InterruptDescriptor,
    pub segment_not_present: InterruptDescriptor,
    pub stack_segment_fault: InterruptDescriptor,
    pub general_protection_fault: InterruptDescriptor,
    pub page_fault: InterruptDescriptor,
    pub _reserved_0: InterruptDescriptor,
    pub x87_float_exception: InterruptDescriptor,
    pub alignment_check: InterruptDescriptor,
    pub machine_check: InterruptDescriptor,
    pub simd_float_exception: InterruptDescriptor,
    pub virtualization_exception: InterruptDescriptor,
    pub controller_protection_exception: InterruptDescriptor,
    pub _reserved_1: [InterruptDescriptor; 6],
    pub hypervisor_injection_exception: InterruptDescriptor,
    pub vmm_communication_exception: InterruptDescriptor,
    pub security_exception: InterruptDescriptor,
    pub _reserved_2: InterruptDescriptor,
    pub interrupts: [InterruptDescriptor; 256 - 32],
}

impl Default for IDT {
    fn default() -> Self {
        Self {
            interrupts: [InterruptDescriptor::default(); 256 - 32],
            _reserved_1: [InterruptDescriptor::default(); 6],
            breakpoint: InterruptDescriptor::default(),
            division_error: InterruptDescriptor::default(),
            debug: InterruptDescriptor::default(),
            nmi: InterruptDescriptor::default(),
            overflow: InterruptDescriptor::default(),
            bound_range_exceeded: InterruptDescriptor::default(),
            invalid_opcode: InterruptDescriptor::default(),
            device_not_available: InterruptDescriptor::default(),
            double_fault: InterruptDescriptor::default(),
            coprocessor_segment_overrun: InterruptDescriptor::default(),
            invalid_tss: InterruptDescriptor::default(),
            segment_not_present: InterruptDescriptor::default(),
            stack_segment_fault: InterruptDescriptor::default(),
            general_protection_fault: InterruptDescriptor::default(),
            page_fault: InterruptDescriptor::default(),
            _reserved_0: InterruptDescriptor::default(),
            x87_float_exception: InterruptDescriptor::default(),
            alignment_check: InterruptDescriptor::default(),
            machine_check: InterruptDescriptor::default(),
            simd_float_exception: InterruptDescriptor::default(),
            virtualization_exception: InterruptDescriptor::default(),
            controller_protection_exception: InterruptDescriptor::default(),
            hypervisor_injection_exception: InterruptDescriptor::default(),
            vmm_communication_exception: InterruptDescriptor::default(),
            security_exception: InterruptDescriptor::default(),
            _reserved_2: InterruptDescriptor::default(),
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InterruptDescriptor {
    offset_1: u16,
    selector: SegmentSelector,
    ist: u8,
    type_attributes: TypeAttributes,
    offset_2: u16,
    offset_3: u32,
    zero: u32,
}

type InterruptFn = extern "x86-interrupt" fn(stack_frame: &InterruptStackFrame);
type InterruptFnErrorCode = extern "x86-interrupt" fn(stack_frame: &InterruptStackFrame, error_code: u64);
type InterruptFnPageFault = extern "x86-interrupt" fn(stack_frame: &InterruptStackFrame, page_fault: PageFaultErrorCode);

impl InterruptDescriptor {
    pub fn new_interrupt(isr: InterruptFn) -> Self {
        let offset = (isr as *const InterruptFn) as u64;
        Self {
            offset_1: offset as u16,
            selector: SegmentSelector(0x8),
            ist: 0,
            type_attributes: TypeAttributes(0x8e),
            offset_2: (offset >> 16) as u16,
            offset_3: (offset >> 32) as u32,
            zero: 0,
        }
    }

    pub fn new_trap(isr: InterruptFn) -> Self {
        let offset = (isr as *const InterruptFn) as u64;
        Self {
            offset_1: offset as u16,
            selector: SegmentSelector(0x8),
            ist: 0,
            type_attributes: TypeAttributes(0x8e),
            offset_2: (offset >> 16) as u16,
            offset_3: (offset >> 32) as u32,
            zero: 0,
        }
    }

    pub fn new_interrupt_error_code(isr: InterruptFnErrorCode) -> Self {
        let offset = (isr as *const InterruptFnErrorCode) as u64;
        Self {
            offset_1: offset as u16,
            selector: SegmentSelector(0x8),
            ist: 0,
            type_attributes: TypeAttributes(0x8e),
            offset_2: (offset >> 16) as u16,
            offset_3: (offset >> 32) as u32,
            zero: 0,
        }
    }

    pub fn new_trap_error_code(isr: InterruptFnErrorCode) -> Self {
        let offset = (isr as *const InterruptFnErrorCode) as u64;
        Self {
            offset_1: offset as u16,
            selector: SegmentSelector(0x8),
            ist: 0,
            type_attributes: TypeAttributes(0x8e),
            offset_2: (offset >> 16) as u16,
            offset_3: (offset >> 32) as u32,
            zero: 0,
        }
    }

    pub fn new_page_fault(isr: InterruptFnPageFault) -> Self {
        let offset = (isr as *const InterruptFnPageFault) as u64;
        Self {
            offset_1: offset as u16,
            selector: SegmentSelector(0x8),
            ist: 0,
            type_attributes: TypeAttributes(0x8e),
            offset_2: (offset >> 16) as u16,
            offset_3: (offset >> 32) as u32,
            zero: 0,
        }
    }
}

bitfield! {
    #[derive(Clone, Copy, Default)]
    pub struct SegmentSelector(u16);
    impl Debug;
    rpl, set_rpl: 1, 0;
    ti, set_ti: 2;
    index, set_index: 15, 3;
}

bitfield! {
    #[derive(Clone, Copy, Default)]
    pub struct TypeAttributes(u8);
    impl Debug;
    gate_type, set_gate_type: 3, 0;
    dpl, set_dpl: 6, 5;
    present, set_present: 7;
}

bitfield! {
    #[derive(Clone, Copy, Default)]
    pub struct PageFaultErrorCode(u64);
    impl Debug;
    pub present, _: 0;
    pub write, _: 1;
    pub user, _: 2;
    pub reserved, _: 3;
    pub instruction, _: 4;
    pub protection_key, _: 5;
    pub shadow_stack, _: 7;
    pub sgx, _: 15;
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}
