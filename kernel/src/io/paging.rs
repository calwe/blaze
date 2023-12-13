use core::marker::PhantomData;

use bitfield::bitfield;

use crate::util::{is_aligned, align_down};
use log::warn;

bitfield! {
    #[derive(Copy, Clone, Default)]
    #[repr(transparent)]
    pub struct PageTableEntry(u64);
    impl Debug;
    pub present, set_present: 0;
    pub read_write, set_read_write: 1;
    pub user, set_user: 2;
    pub write_through, set_write_through: 3;
    pub cache_disable, set_cache_disable: 4;
    pub accessed, set_accessed: 5;
    pub dirty, set_dirty: 6;
    pub size, set_size: 7;
    pub granularity, set_granularity: 8;
    pub availible, set_availible: 11, 9;
    pub address, set_address: 51, 12;
    pub availible_2, set_availible_2: 58, 52;
    pub protection_key, set_protection_key: 62, 59;
    pub execute_disable, set_execute_disable: 63;
}

pub trait PageSize {
    const SIZE: u64;
}

pub struct Size4KiB;
pub struct Size2MiB;
pub struct Size1GiB;

impl PageSize for Size4KiB {
    const SIZE: u64 = 4 * 1024;
}

impl PageSize for Size2MiB {
    const SIZE: u64 = 2 * 1024 * 1024;
}

impl PageSize for Size1GiB {
    const SIZE: u64 = 1024 * 1024 * 1024;
}

pub struct Page<S: PageSize> {
    pub address: u64,
    // compiler doesnt like unused types
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    pub fn from_address(addr: u64) -> Self {
        if !is_aligned(addr, S::SIZE) {
            let a_addr = align_down(addr, S::SIZE);
            warn!("Address 0x{addr:x} not aligned, creating page at 0x{a_addr:x}");
            Self {
                address: a_addr,
                size: PhantomData,
            }
        } else {
            Self {
                address: addr,
                size: PhantomData
            }
        }
    }
}
