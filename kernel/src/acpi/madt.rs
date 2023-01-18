use core::ptr::from_raw_parts;

use crate::trace;

use super::rsdt::ACPISDTHeader;

#[repr(C, packed)]
pub struct MADT {
    header: ACPISDTHeader,
    local_apic_address: u32,
    flags: u32,
    entries: [u8],
}

impl MADT {
    pub fn from_addr(addr: u32) -> *const MADT {
        let header = unsafe { *(addr as *const ACPISDTHeader) };
        let len = header.length;
        trace!("MADT Length: {}", len);
        let entries = (header.length - core::mem::size_of::<ACPISDTHeader>() as u32 - 8) / 4;
        from_raw_parts(addr as *const (), entries as usize)
    }
}
