use core::ptr::from_raw_parts;

use alloc::string::{String, ToString};

use crate::{trace, warn};

use super::{hpet::HPET, madt::MADT};

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Default)]
/// The ACPISDTHeader is the header for all ACPI tables. It is used to identify the table and to
/// calculate the checksum.
pub struct ACPISDTHeader {
    signature: [u8; 4],
    /// The length of the table, including the header.
    pub length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[repr(C, packed)]
/// The RSDT is the main System Description Table.
pub struct RSDT {
    header: ACPISDTHeader,
    entries: [u32],
}

impl ACPISDTHeader {
    /// Returns the signature as a string.
    pub fn signature(&self) -> String {
        String::from_utf8_lossy(&self.signature).to_string()
    }
}

impl RSDT {
    /// Creates a new RSDT from a given address.
    pub fn from_addr(addr: u32) -> *const RSDT {
        let header = unsafe { *(addr as *const ACPISDTHeader) };
        let entries = (header.length - core::mem::size_of::<ACPISDTHeader>() as u32) / 4;
        from_raw_parts(addr as *const (), entries as usize)
    }

    /// Creates an iterator over the entries of the RSDT.
    pub fn entries(&self) -> RSDTIterator {
        RSDTIterator {
            rsdt: self,
            index: 0,
        }
    }

    /// Returns the first entry with the given signature.
    fn get_common(&self, signature: &str) -> Option<u32> {
        for entry in self.entries() {
            let entry_deref = unsafe { *entry };
            if entry_deref.signature() == signature {
                // get mem address of entry
                let addr = entry as u32;
                trace!("Found {} at {:#x}", signature, addr);
                return Some(addr);
            }
        }
        warn!("Could not find {}", signature);
        None
    }

    /// Returns the MADT if it exists.
    pub fn get_madt(&self) -> Option<*const MADT> {
        let addr = self.get_common("APIC")?;
        Some(MADT::from_addr(addr))
    }

    /// Returns the HPET if it exists.
    pub fn get_hpet(&self) -> Option<&'static HPET> {
        let addr = self.get_common("HPET")?;
        Some(HPET::new(addr))
    }
}

/// The RSDTIterator is used to iterate over the entries of the RSDT. It returns ACPISDTHeaders.
pub struct RSDTIterator {
    rsdt: *const RSDT,
    index: usize,
}

impl Iterator for RSDTIterator {
    type Item = *const ACPISDTHeader;

    fn next(&mut self) -> Option<Self::Item> {
        let rsdt = unsafe { &*self.rsdt };
        let entries = (rsdt.header.length - core::mem::size_of::<ACPISDTHeader>() as u32) / 4;
        if self.index >= entries as usize {
            return None;
        }
        let entry = rsdt.entries[self.index] as *const ACPISDTHeader;
        self.index += 1;
        Some(entry)
    }
}
