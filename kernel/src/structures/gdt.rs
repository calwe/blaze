use core::mem::size_of;

use bitfield::bitfield;
use core::arch::asm;
use log::debug;

use super::tss::TSS;

#[repr(C, packed)]
pub struct GDTR {
    size: u16,
    offset: u64,
}

impl GDTR {
    pub fn new(gdt: &GDT) -> Self {
        Self {
            size: size_of::<GDT>() as u16,
            offset: (gdt as *const GDT) as u64
        }
    }

    pub fn load(&self) {
        unsafe {
            asm!("lgdt [{}]", in(reg) self)
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct GDT {
    null: SegmentDescriptor32,
    kernel_code: SegmentDescriptor32,
    kernel_data: SegmentDescriptor32,
    user_code: SegmentDescriptor32,
    user_data: SegmentDescriptor32,
    tss: SegmentDescriptor64,
}

impl GDT {
    pub fn with_tss(tss: &TSS) -> Self {
        debug!("Size of Descriptor: {}", size_of::<SegmentDescriptor32>());
        Self {
            null: SegmentDescriptor32::null(),
            kernel_code: SegmentDescriptor32::kernel_code(),
            kernel_data: SegmentDescriptor32::kernel_data(),
            user_code: SegmentDescriptor32::user_code(),
            user_data: SegmentDescriptor32::user_data(),
            tss: SegmentDescriptor64::tss(tss),
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct SegmentDescriptor32 {
    limit: u16,
    base_1: u16,
    base_2: u8,
    access_byte: AccessByte,
    flags_limit: FlagsLimit,
    base_3: u8,
}

impl SegmentDescriptor32 {
    pub fn null() -> Self {
        Self {
            limit: 0,
            base_1: 0,
            base_2: 0,
            access_byte: AccessByte(0),
            flags_limit: FlagsLimit(0),
            base_3: 0,
        }
    }

    pub fn kernel_code() -> Self {
        Self {
            limit: 0xFFFF,
            base_1: 0,
            base_2: 0,
            access_byte: AccessByte(0x9A),
            flags_limit: FlagsLimit(0xAF),
            base_3: 0,
        }
    }

    pub fn kernel_data() -> Self {
        Self {
            limit: 0xFFFF,
            base_1: 0,
            base_2: 0,
            access_byte: AccessByte(0x92),
            flags_limit: FlagsLimit(0xCF),
            base_3: 0,
        }
    }

    pub fn user_code() -> Self {
        Self {
            limit: 0xFFFF,
            base_1: 0,
            base_2: 0,
            access_byte: AccessByte(0xFA),
            flags_limit: FlagsLimit(0xAF),
            base_3: 0,
        }
    }

    pub fn user_data() -> Self {
        Self {
            limit: 0xFFFF,
            base_1: 0,
            base_2: 0,
            access_byte: AccessByte(0xF2),
            flags_limit: FlagsLimit(0xCF),
            base_3: 0,
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct SegmentDescriptor64 {
    limit: u16,
    base_1: u16,
    base_2: u8,
    access_byte: SystemAccessByte,
    flags_limit: FlagsLimit,
    base_3: u8,
    base_4: u32,
    reserved: u32,
}

impl SegmentDescriptor64 {
    pub fn tss(tss: &TSS) -> Self {
        let base = (tss as *const TSS) as u64;
        let limit = size_of::<TSS>();
        Self {
            limit: limit as u16,
            base_1: base as u16,
            base_2: (base >> 16) as u8,
            access_byte: SystemAccessByte(0x89),
            flags_limit: FlagsLimit((limit >> 20) as u8),
            base_3: (base >> 24) as u8,
            base_4: (base >> 32) as u32,
            reserved: 0,
        }
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AccessByte(u8);
    impl Debug;
    pub accessed, set_accessed: 0;
    pub read_write, set_read_write: 1;
    pub direction, set_direction: 2;
    pub executable, set_executable: 3;
    one, _: 4;
    pub dpl, set_dpl: 6, 5;
    pub present, set_present: 7;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct SystemAccessByte(u8);
    impl Debug;
    pub descriptor_type, set_descriptor_type: 3, 0;
    zero, _: 4;
    pub dpl, set_dpl: 6, 5;
    pub present, set_present: 7;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FlagsLimit(u8);
    impl Debug;
    reserved, _: 0;
    long_mode, set_long_mode: 1;
    size, set_size: 2;
    granularity, set_granularity: 3;
    limit, set_limit: 7, 4;
}
