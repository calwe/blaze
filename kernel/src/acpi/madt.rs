use core::ptr::from_raw_parts;

use num_derive::FromPrimitive;

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

impl MADTEntry {
    /// Returns the type of the entry.
    pub fn get_type(&self) -> Option<MADTEntryTypes> {
        match self.entry_type {
            0 => Some(MADTEntryTypes::ProcessorLocalAPIC(ProcessorLocalAPIC::new(
                &self.data,
            ))),
            1 => Some(MADTEntryTypes::IOAPIC(IOAPIC::new(&self.data))),
            2 => Some(MADTEntryTypes::InterruptSourceOverride),
            3 => Some(MADTEntryTypes::IOAPICNMISource),
            4 => Some(MADTEntryTypes::LocalAPICNMI),
            5 => Some(MADTEntryTypes::LocalAPICAddressOverride),
            9 => Some(MADTEntryTypes::ProcessorLocalx2APIC),
            _ => None,
        }
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

// ----------------------------------------------------------------------------
// - MADT Entry Types
// ----------------------------------------------------------------------------

/// Each entry has its own type that describes how the data is laid out.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MADTEntryTypes {
    /// This entry describes a single logival processor and its interrupt controller.
    ProcessorLocalAPIC(ProcessorLocalAPIC),
    /// I/O APIC
    IOAPIC(IOAPIC),
    /// Interrupt Source Override. This entry describes an interrupt source that is
    /// mapped to a different interrupt vector.
    InterruptSourceOverride,
    /// Specifies the interrupt sources that are used for Non-Maskable Interrupts.
    IOAPICNMISource,
    /// Configure these with the LINT0 and LINT1 entries in the Local vector
    /// table of the relevant processor(')s(') local APIC.
    LocalAPICNMI,
    /// Provides 64-bit systems woth an override of the physical address of the
    /// local APIC. There can only be one of these entries in the MADT. If this
    /// entry is present, the 32-Bit Local APIC Address stored in the MADT header
    /// should be ignored.
    LocalAPICAddressOverride,
    /// Represents a physical processo and its Local x2APIC. Identical to the
    /// Local APIC, used only when that strucct cannot hold the required values.
    ProcessorLocalx2APIC = 9,
}

/// This entry describes a single logical processor and its interrupt controller.
#[derive(Debug, Clone, Copy)]
pub struct ProcessorLocalAPIC {
    /// ACPI processor ID
    processor_id: u8,
    /// ACPI processor ID of the processor's parent
    apic_id: u8,
    /// Flags
    flags: u32,
}

impl ProcessorLocalAPIC {
    /// Creates a new ProcessorLocalAPIC from the given data.
    pub fn new(data: &[u8]) -> Self {
        Self {
            processor_id: data[0],
            apic_id: data[1],
            flags: u32::from_le_bytes([data[2], data[3], data[4], data[5]]),
        }
    }
}

/// I/O APIC
#[derive(Debug, Clone, Copy)]
pub struct IOAPIC {
    /// I/O APIC ID
    ioapic_id: u8,
    /// Reserved
    reserved: u8,
    /// I/O APIC Address
    ioapic_address: u32,
    /// Global System Interrupt Base
    global_system_interrupt_base: u32,
}

impl IOAPIC {
    /// Creates a new IOAPIC from the given data.
    pub fn new(data: &[u8]) -> Self {
        Self {
            ioapic_id: data[0],
            reserved: data[1],
            ioapic_address: u32::from_le_bytes([data[2], data[3], data[4], data[5]]),
            global_system_interrupt_base: u32::from_le_bytes([data[6], data[7], data[8], data[9]]),
        }
    }
}

// TODO: Implement the rest of the MADT entry types.
