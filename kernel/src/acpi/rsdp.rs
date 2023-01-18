use alloc::string::{String, ToString};
use limine::LimineRsdpResponse;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct RSDPDescriptor {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

impl RSDPDescriptor {
    ///
    pub fn from_rsdp_response(response: &LimineRsdpResponse) -> Self {
        unsafe { *(response.address.as_ptr().unwrap() as *const RSDPDescriptor) }
    }

    /// Returns the RSDP signature as a string.
    pub fn signature(&self) -> String {
        String::from_utf8_lossy(&self.signature).to_string()
    }

    /// Returns an extended rsdp if the revision is 2.
    pub fn extended_rsdp(&self) -> Option<RSDPDescriptor20> {
        if self.revision == 2 {
            Some(unsafe { core::mem::transmute_copy(self) })
        } else {
            None
        }
    }

    /// Checks if the RSDP is valid.
    pub fn checksum(&self) -> bool {
        let mut sum: u8 = 0;
        for i in 0..20 {
            sum = sum.wrapping_add(unsafe { *(self as *const RSDPDescriptor as *const u8).add(i) });
        }
        sum == 0
    }

    /// Returns the OEMID as a string.
    pub fn oem_id(&self) -> String {
        String::from_utf8_lossy(&self.oem_id).to_string()
    }

    /// Returns the address
    pub fn rsdt_address(&self) -> u32 {
        self.rsdt_address
    }
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

impl RSDPDescriptor20 {
    /// Returns the RSDP signature as a string.
    pub fn signature(&self) -> String {
        String::from_utf8_lossy(&self.rsdp.signature).to_string()
    }
}
