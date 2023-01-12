//! Tools for loading ELF binaries.

use crate::{trace, error};

const ELF64_MAGIC: u32 = 0x7f
                            | ('E' as u32) << 8
                            | ('L' as u32) << 16 
                            | ('F' as u32) << 24;

type Elf64_Addr = u64;
type Elf64_Off = u64;
type Elf64_Half = u16;
type Elf64_Word = u32;
type Elf64_Sword = u32;
type Elf64_Xword = u64;
type Elf64_Sxword = u64;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct E_Ident {
    magic: u32,
    ei_class: u8,
    ei_data: u8,
    ei_version: u8,
    ei_osabi: u8,
    ei_abiversion: u8,
    ei_pad0: u32,
    ei_pad1: u16,
    ei_nident: u8,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
/// The file header is located at the beggining of the file,
/// and is used to locate the other parts of the file.
pub struct Elf64_Ehdr {
    e_ident: E_Ident, // ELF Ident
    e_type: Elf64_Half, // Object file type
    e_machine: Elf64_Half, // Machine type
    e_version: Elf64_Word, // Object file version
    e_entry: Elf64_Addr, // Entry point address
    e_phoff: Elf64_Off, // Program header offset
    e_shoff: Elf64_Off, // Section header offset
    e_flags: Elf64_Word, // Processor-specifc flags
    e_ehsize: Elf64_Half, // ELF header size
    e_phentsize: Elf64_Half, // Size of program header entry
    e_phnum: Elf64_Half, // Number of program header entries
    e_shentsize: Elf64_Half, // Size of section header entry
    e_shnum: Elf64_Half, // Number of section header entries
    e_shstrnfx: Elf64_Half // Section name string table index
}

/// Loads an elf into memory given its address
pub fn load_elf_at_addr(addr: u64) {
    trace!("Loading ELF stored at 0x{:x}", addr);
    let elf_header = unsafe {
        *(addr as *const Elf64_Ehdr)
    };

    let magic = elf_header.e_ident.magic;
    if magic != ELF64_MAGIC {
        error!("Invalid magic! (0x{:x} != 0x{:x})", magic, ELF64_MAGIC);
        return;
    }

    // TODO: https://wiki.osdev.org/ELF
}