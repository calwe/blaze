use core::ptr::from_raw_parts;

use crate::trace;
use core::fmt::Debug;

use super::rsdt::ACPISDTHeader;

#[repr(C, packed)]
pub struct MADT {
    header: ACPISDTHeader,
    local_apic_address: u32,
    flags: u32,
    entries: [u8],
}

#[repr(C, packed)]
pub struct MADTEntry {
    entry_type: u8,
    length: u8,
    data: [u8],
}

pub struct MADTIterator {
    madt: *const MADT,
    index: usize,
}

impl Iterator for MADTIterator {
    type Item = &'static MADTEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let madt = unsafe { &*self.madt };
        if (self.index + 1) >= madt.entries.len() {
            return None;
        }
        let entries = unsafe { madt.entries.as_ptr().add(self.index) };
        let len = madt.entries[self.index + 1] as usize;
        let entry = core::ptr::from_raw_parts(entries as *const (), len) as *const MADTEntry;
        self.index += len;
        Some(unsafe { &*entry })
    }
}

impl Debug for MADT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let apic_addr = self.local_apic_address;
        let flags = self.flags;
        let entries = &self.entries;
        f.debug_struct("MADT")
            .field("header", &self.header)
            .field("local_apic_address", &apic_addr)
            .field("flags", &flags)
            .field("entries", &entries)
            .finish()
    }
}

impl Debug for MADTEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let entry_type = self.entry_type;
        let length = self.length;
        let data = &self.data;
        f.debug_struct("MADTEntry")
            .field("entry_type", &entry_type)
            .field("length", &length)
            .field("data", &data)
            .finish()
    }
}

impl MADT {
    pub fn from_addr(addr: u32) -> *const MADT {
        let header = unsafe { *(addr as *const ACPISDTHeader) };
        let entries = (header.length - core::mem::size_of::<ACPISDTHeader>() as u32 - 8) / 4;
        from_raw_parts(addr as *const (), entries as usize)
    }

    pub fn entries(&self) -> MADTIterator {
        MADTIterator {
            madt: self,
            index: 0,
        }
    }
}
