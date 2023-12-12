#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TSS {
    _reserved_1: u32,
    pub rsp: [u64; 3],
    _reserved_2: u64,
    pub ist: [u64; 7],
    _reserved_3: u64,
    _reserved_4: u16,
    pub iopb: u16,
}
