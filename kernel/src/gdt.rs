pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const KERNEL_STACK_INDEX: u16 = 0;

use core::arch::asm;

use lazy_static::lazy_static;
use x86_64::{
    structures::{
        gdt::{Descriptor, DescriptorFlags, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::trace;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.privilege_stack_table[KERNEL_STACK_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // 16-bit code segment
        gdt.add_entry(Descriptor::UserSegment((DescriptorFlags::COMMON | DescriptorFlags::EXECUTABLE).bits()));
        // 16-bit data segment
        gdt.add_entry(Descriptor::UserSegment((DescriptorFlags::COMMON).bits()));
        // 32-bit code segment
        gdt.add_entry(Descriptor::UserSegment(DescriptorFlags::KERNEL_CODE32.bits()));
        // 32-bit data segment
        gdt.add_entry(Descriptor::UserSegment(DescriptorFlags::KERNEL_DATA.bits()));
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        gdt.add_entry(Descriptor::kernel_data_segment());
        gdt.add_entry(Descriptor::user_code_segment());
        gdt.add_entry(Descriptor::user_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

pub fn init() {
    trace!("Initializing GDT");

    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();

    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }

    trace!("GDT initialized");
}
