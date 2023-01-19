//! Global Descriptor Table

/// The index of the double fault stack in the TSS.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
/// The index of the kernel stack in the TSS.
pub const KERNEL_STACK_INDEX: u16 = 0;

use lazy_static::lazy_static;
use x86_64::{
    structures::{
        gdt::{Descriptor, DescriptorFlags, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::{info, trace};

lazy_static! {
    // In 64-bit mode, the TSS is used to store stack pointers for privilege level changes
    // as well as interrupt stacks.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.privilege_stack_table[KERNEL_STACK_INDEX as usize] = {
            // All our stacks are 5 pages long (20 KiB)
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // All our stacks are 5 pages long (20 KiB)
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
        // To use the terminal provided by Limine, we need a specific GDT layout:
        // 1. Null descriptor (added by default)
        // 2. 16-bit code segment
        // 3. 16-bit data segment
        // 4. 32-bit code segment
        // 5. 32-bit data segment
        // 6. 64-bit code segment
        // 7. 64-bit data segment
        // We also need user mode segments, and a TSS segment.

        // 16-bit code segment
        gdt.add_entry(Descriptor::UserSegment((DescriptorFlags::COMMON | DescriptorFlags::EXECUTABLE).bits()));
        // 16-bit data segment
        gdt.add_entry(Descriptor::UserSegment((DescriptorFlags::COMMON).bits()));
        // 32-bit code segment
        gdt.add_entry(Descriptor::UserSegment(DescriptorFlags::KERNEL_CODE32.bits()));
        // 32-bit data segment
        gdt.add_entry(Descriptor::UserSegment(DescriptorFlags::KERNEL_DATA.bits()));
        // 64-bit code segment
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // 64-bit data segment
        gdt.add_entry(Descriptor::kernel_data_segment());
        // User mode code segment
        gdt.add_entry(Descriptor::user_code_segment());
        // User mode data segment
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

/// Initializes the Global Descriptor Table and Task State Segment.
pub fn init() {
    trace!("Initializing GDT");

    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();

    // SAFETY: We *must* pass a valid code selector to `CS::set_reg`,
    // aswell as a valid TSS selector to `load_tss`.
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }

    info!("GDT initialized");
}
