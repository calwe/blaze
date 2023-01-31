use core::ptr::from_raw_parts;

use core::fmt::Debug;

use crate::{trace, warn};
use bitfield::bitfield;
use spin::Mutex;

use super::rsdt::ACPISDTHeader;

#[repr(C, packed)]
/// Multiple APIC Description Table
/// The MADT is used to describe the APICs in the system.
pub struct MADT {
    header: ACPISDTHeader,
    local_apic_address: u32,
    flags: u32,
    entries: [u8],
}

#[repr(C, packed)]
/// The entries in the MADT.
pub struct MADTEntry {
    entry_type: u8,
    length: u8,
    data: [u8],
}

/// An iterator over the entries of the MADT.
/// The entries are variable length, so this iterator makes it far easier
/// to access each.
pub struct MADTIterator {
    madt: *const MADT,
    index: usize,
}

impl Iterator for MADTIterator {
    type Item = &'static MADTEntry;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Check the MADT checksum
        let madt = unsafe { &*self.madt };
        // Check if we are at the end of the entries
        if (self.index + 1) >= madt.entries.len() {
            return None;
        }
        // The entry is offset by the index from the start of the entries.
        let entries = unsafe { madt.entries.as_ptr().add(self.index) };
        // The length of the entry is stored in the next byte.
        let len = madt.entries[self.index + 1] as usize;
        // `madt.entries` is a DST, so we need to use `from_raw_parts` to specify the length for the pointer.
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
            2 => Some(MADTEntryTypes::InterruptSourceOverride(
                InterruptSourceOverride::new(&self.data),
            )),
            3 => Some(MADTEntryTypes::IOAPICNMISource),
            4 => Some(MADTEntryTypes::LocalAPICNMI),
            5 => Some(MADTEntryTypes::LocalAPICAddressOverride),
            9 => Some(MADTEntryTypes::ProcessorLocalx2APIC),
            _ => {
                warn!("Unknown MADT entry type: {}", self.entry_type);
                None
            }
        }
    }
}

impl MADT {
    /// Gets the MADT from the given address.
    pub fn from_addr(addr: u32) -> *const MADT {
        // The header is at the start of the table.
        let header = unsafe { *(addr as *const ACPISDTHeader) };
        // Then we can figure out how many entries there are.
        let entries = header.length - core::mem::size_of::<ACPISDTHeader>() as u32 - 8;
        // The final field is a DST, so we need to use `from_raw_parts` to specify the length of the array.
        from_raw_parts(addr as *const (), entries as usize)
    }

    /// Returns an iterator over the entries in the MADT.
    pub fn entries(&self) -> MADTIterator {
        MADTIterator {
            madt: self,
            index: 0,
        }
    }

    /// Returns the address of the local APIC.
    pub fn local_apic_address(&self) -> u32 {
        self.local_apic_address
    }

    /// Write 4 bytes to an LAPIC reg
    pub fn write_apic_reg(&self, reg: u32, value: u32) {
        let register_location = self.local_apic_address + reg;
        unsafe {
            core::ptr::write_volatile(register_location as *mut u32, value);
        }
    }

    /// Read 4 bytes from an LAPIC reg
    pub fn read_apic_reg(&self, reg: u32) -> u32 {
        let register_location = self.local_apic_address + reg;
        unsafe { core::ptr::read_volatile(register_location as *const u32) }
    }

    // FIXME: This is a BAD hack to test the APIC. The APIC address should NOT be hardcoded.
    /// Write 4 bytes to a LAPIC reg, hardcoded location
    #[allow(non_snake_case)]
    pub fn write_apic_reg_HACK(reg: u32, value: u32) {
        let register_location = 0xFEE00000 + reg;
        unsafe {
            core::ptr::write_volatile(register_location as *mut u32, value);
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
    InterruptSourceOverride(InterruptSourceOverride),
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
#[allow(dead_code)]
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

/// Global list of free interrupt sources, where a 1 is taken, 0 is free
pub static FREE_INTERRUPT_SOURCES: Mutex<InterruptSources> =
    // 0-2 and 8 are not free by default.
    Mutex::new(InterruptSources(1 | 1 << 1 | 1 << 2 | 1 << 8));

/// The first IOAPIC in the system
// TODO: Do we need to change how we are accessing this?
//          we need to keep in mind that systems can have multiple IOAPICs
pub static IOAPIC_0: Mutex<Option<IOAPIC>> = Mutex::new(None);

/// list of free interrupt sources, where a 1 is taken, 0 is free
pub struct InterruptSources(u64);

impl InterruptSources {
    /// Set the bit of a given irq
    pub fn set_irq(&mut self, irq: u8) {
        self.0 |= 1 << irq;
    }

    /// Clear the bit of a given irq
    pub fn clear_irq(&mut self, irq: u8) {
        self.0 &= 0 << irq;
    }

    /// Return whether or not a irq is free
    pub fn get_irq(&mut self, irq: u8) -> bool {
        self.0 & (1 << irq) == 0
    }
}

/// I/O APIC
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct IOAPIC {
    /// I/O APIC ID
    ioapic_id: u8,
    /// Reserved
    reserved: u8,
    /// I/O APIC Address
    pub ioapic_address: u32,
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

    /// Read from IOAPIC register
    pub fn read(&self, reg: u8) -> u32 {
        unsafe {
            core::ptr::write_volatile(self.ioapic_address as *mut u32, reg as u32);
            core::ptr::read_volatile((self.ioapic_address + 0x10) as *mut u32)
        }
    }

    /// Write to IOAPIC register
    pub fn write(&self, reg: u8, value: u32) {
        unsafe {
            core::ptr::write_volatile(self.ioapic_address as *mut u32, reg as u32);
            core::ptr::write_volatile((self.ioapic_address + 0x10) as *mut u32, value);
        }
    }

    /// Read table entry
    pub fn read_table_entry(&self, index: u8) -> u64 {
        let low = self.read(index * 2 + 0x10);
        let high = self.read(index * 2 + 1 + 0x10);
        u64::from_le_bytes([
            low as u8,
            (low >> 8) as u8,
            (low >> 16) as u8,
            (low >> 24) as u8,
            high as u8,
            (high >> 8) as u8,
            (high >> 16) as u8,
            (high >> 24) as u8,
        ])
    }

    /// Write table entry
    pub fn write_table_entry(&self, index: u8, entry: IOREDTBL) {
        let value = entry.0;
        let low = u32::from_le_bytes([
            value as u8,
            (value >> 8) as u8,
            (value >> 16) as u8,
            (value >> 24) as u8,
        ]);
        let high = u32::from_le_bytes([
            (value >> 32) as u8,
            (value >> 40) as u8,
            (value >> 48) as u8,
            (value >> 56) as u8,
        ]);
        self.write(index * 2 + 0x10, low);
        self.write(index * 2 + 1 + 0x10, high);
    }

    /// Write default table entry, with given vector for redirection
    pub fn standard_table_entry(&self, irq: u8, vector: u8) {
        trace!("Mapping IRQ{irq} to IDT[{vector}]");
        let mut standard_entry = IOREDTBL(0);
        standard_entry.set_vector(vector as u64);
        FREE_INTERRUPT_SOURCES.lock().set_irq(irq);
        self.write_table_entry(irq, standard_entry);
    }
}

bitfield! {
    /// I/O Redirection Table Entry
    pub struct IOREDTBL(u64);
    impl Debug;
    /// The interrupt vector that will be raised to the specified processor.
    pub vector, set_vector: 7, 0;
    /// The delivery mode of the interrupt.
    /// 000: Fixed
    /// 001: Lowest Priority
    /// 010: SMI
    /// 100: NMI
    /// 101: INIT
    /// 111: ExtINT
    pub delivery_mode, set_delivery_mode: 10, 8;
    /// The destination mode of the interrupt.
    /// 0: Physical
    /// 1: Logical
    pub destination_mode, set_destination_mode: 11;
    /// The delivery status of the interrupt.
    pub delivery_status, set_delivery_status: 12;
    /// The polarity of the interrupt.
    /// 0: Active High
    /// 1: Active Low
    pub polarity, set_polarity: 13;
    /// The remote IRR of the interrupt.
    pub remote_irr, set_remote_irr: 14;
    /// The trigger mode of the interrupt.
    /// 0: Edge
    /// 1: Level
    pub trigger_mode, set_trigger_mode: 15;
    /// The interrupt mask.
    /// 0: Unmasked
    /// 1: Masked
    pub mask, set_mask: 16;
    reserved, _: 55, 17;
    /// The destination field of the interrupt.
    pub destination, set_destination: 63, 56;
}

#[derive(Debug, Clone, Copy)]
/// Interrupt Source Override
#[allow(dead_code)]
pub struct InterruptSourceOverride {
    /// Bus
    bus: u8,
    /// Source
    source: u8,
    /// GSI
    global_system_interrupt: u32,
    /// Flags
    flags: u16,
}

impl InterruptSourceOverride {
    /// Create a new ISO from data
    pub fn new(data: &[u8]) -> Self {
        Self {
            bus: data[0],
            source: data[1],
            global_system_interrupt: u32::from_le_bytes([data[2], data[3], data[4], data[5]]),
            flags: u16::from_le_bytes([data[6], data[7]]),
        }
    }
}

// TODO: Implement the rest of the MADT entry types.
