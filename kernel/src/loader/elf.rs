//! Tools for loading ELF binaries.

use core::ptr;

use x86_64::{align_up, structures::paging::mapper::MapToError, VirtAddr};

use crate::{
    error,
    memory::{self, allocator::allocate_of_size, BootInfoFrameAllocator},
    trace, warn, MEMORY_MAP,
};

type Elf64_Addr = u64;
type Elf64_Off = u64;
type Elf64_Half = u16;
type Elf64_Word = u32;
type Elf64_Sword = u32;
type Elf64_Xword = u64;
type Elf64_Sxword = u64;

const ELF64_MAGIC: u32 = 0x7f | ('E' as u32) << 8 | ('L' as u32) << 16 | ('F' as u32) << 24;
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const EM_X86_64: u16 = 62;
const EV_CURRENT: u32 = 1;
const ET_REL: u16 = 1;
const ET_EXEC: u16 = 2;
const ET_DYN: u16 = 3;
const PT_LOAD: Elf64_Word = 1;

const DEFAULT_STACK_SIZE: u64 = 1024 * 100; // 100KiB

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
    e_ident: E_Ident,        // ELF Ident
    e_type: Elf64_Half,      // Object file type
    e_machine: Elf64_Half,   // Machine type
    e_version: Elf64_Word,   // Object file version
    e_entry: Elf64_Addr,     // Entry point address
    e_phoff: Elf64_Off,      // Program header offset
    e_shoff: Elf64_Off,      // Section header offset
    e_flags: Elf64_Word,     // Processor-specifc flags
    e_ehsize: Elf64_Half,    // ELF header size
    e_phentsize: Elf64_Half, // Size of program header entry
    e_phnum: Elf64_Half,     // Number of program header entries
    e_shentsize: Elf64_Half, // Size of section header entry
    e_shnum: Elf64_Half,     // Number of section header entries
    e_shstrnfx: Elf64_Half,  // Section name string table index
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
/// In executable and shared object files, sections are grouped into segments
/// for loading. The program header table contains a list of entries describing
/// each segment.
pub struct Elf64_Phdr {
    p_type: Elf64_Word,    // type of segment
    p_flags: Elf64_Word,   // Segment attributes
    p_offset: Elf64_Off,   // Offset in file
    p_vaddr: Elf64_Addr,   // Virtual addr in memory
    p_paddr: Elf64_Addr,   // Reserved
    p_filesz: Elf64_Xword, // Size of segment in file
    p_memsz: Elf64_Xword,  // Size of segment in memory
    p_align: Elf64_Xword,  // Alignment of segment
}

/// Loads an elf into memory given its address
pub fn load_elf_at_addr(addr: u64) -> Result<(u64, u64), ()> {
    trace!("Loading ELF stored at 0x{:x}", addr);
    let elf_header = unsafe { *(addr as *const Elf64_Ehdr) };

    if !check_elf_support(&elf_header) {
        return Err(());
    }

    let etype = elf_header.e_type;
    trace!("Type: {}", etype);

    let phnum = elf_header.e_phnum;
    let phoff = elf_header.e_phoff;
    let phentsize = elf_header.e_phentsize;
    trace!("{phnum} entries * {phentsize}B | 0x{phoff:x}");

    let mmap_response = MEMORY_MAP
        .get_response()
        .get()
        .expect("Bootloader did not respond to memory map request.");
    let mut mapper = unsafe { memory::init(VirtAddr::new(0)) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&mmap_response) };

    let mut stack_start = 0;
    for i in 0..phnum {
        let phaddr = (addr + phoff) + (phentsize * i) as u64;
        let program_header = unsafe { *(phaddr as *const Elf64_Phdr) };
        // we only want to load segments marked as loadable
        if program_header.p_type == PT_LOAD {
            trace!("Allocating memory for program header");
            trace!("{:?}", program_header);

            // we then need to allocate the memory for the segment
            let start = program_header.p_vaddr;
            let size = program_header.p_memsz;
            match allocate_of_size(&mut mapper, &mut frame_allocator, start, size, true) {
                Ok(()) => trace!("Succesfully allocated"),
                Err(MapToError::ParentEntryHugePage) => warn!("Already allocted in huge page"),
                Err(MapToError::PageAlreadyMapped(e)) => warn!("Already mapped {e:?}"),
                Err(e) => {
                    error!("Allocation failed: {e:?}");
                    return Err(());
                }
            };

            // update the start of our stack, as we need to allocate it after the program
            stack_start = start + size;

            // find where the segment is in the file
            let poffset = program_header.p_offset;
            let paddr = addr + poffset;
            trace!("Loading segment from {paddr:x}");

            // copy the segment from the file to the memory
            let src = paddr as *const u8;
            let dst = start as *mut u8;
            let size = program_header.p_memsz;
            unsafe {
                ptr::copy_nonoverlapping(src, dst, size as usize);
            }
        }
    }

    // align our stack to the page boundary
    stack_start = align_up(stack_start, 4096);

    // then we need to map the stack for the program
    trace!("Allocating stack for program");
    match allocate_of_size(
        &mut mapper,
        &mut frame_allocator,
        stack_start,
        DEFAULT_STACK_SIZE,
        true,
    ) {
        Ok(()) => trace!("Succesfully allocated"),
        Err(MapToError::ParentEntryHugePage) => warn!("Already allocted in huge page"),
        Err(MapToError::PageAlreadyMapped(e)) => warn!("Already mapped {e:?}"),
        Err(e) => {
            error!("Allocation failed: {e:?}");
            return Err(());
        }
    };

    // the stack grows downwards, so we need to find the end.
    // since we map the stack in pages, we need to align it up to the page size
    let stack_end = align_up(stack_start + DEFAULT_STACK_SIZE, 4096);
    trace!("Stack end: 0x{stack_end:x}");

    let entry = elf_header.e_entry;
    trace!("ELF Loaded. Entry point 0x{entry:x}");
    Ok((entry, stack_end))
    // TODO: https://wiki.osdev.org/ELF
}

fn check_elf_support(elf_header: &Elf64_Ehdr) -> bool {
    let magic = elf_header.e_ident.magic;
    if magic != ELF64_MAGIC {
        error!("Invalid magic! (0x{:x} != 0x{:x})", magic, ELF64_MAGIC);
        return false;
    }
    let class = elf_header.e_ident.ei_class;
    if class != ELFCLASS64 {
        error!("Unsupported class {class}.");
        return false;
    }
    let endianness = elf_header.e_ident.ei_data;
    if endianness != ELFDATA2LSB {
        error!("Unsupported endianness {endianness}.");
        return false;
    }
    let machine = elf_header.e_machine;
    if machine != EM_X86_64 {
        error!("Unsupported machine arch {machine}.");
        return false;
    }
    let version = elf_header.e_version;
    if version != EV_CURRENT {
        error!("Unsupported ELF version {version}");
        return false;
    }
    let etype = elf_header.e_type;
    if etype != ET_REL && etype != ET_EXEC && etype != ET_DYN {
        error!("Unsupported ELF type {etype}");
        return false;
    }
    true
}
