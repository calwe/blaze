#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct RSDPDescriptor {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct RSDPDescriptor20 {
    rsdp: RSDPDescriptor,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}
