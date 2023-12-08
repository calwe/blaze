use multiboot2_header::{HeaderTagISA, builder::{HeaderBuilder, InformationRequestHeaderTagBuilder}, HeaderTagFlag, MbiTagType};
use std::{fs::File, io::Write};

fn main() {
    let mut file = File::create("mbheader/multiboot2_header.bin").unwrap();
    let header = HeaderBuilder::new(HeaderTagISA::I386)
        .information_request_tag(InformationRequestHeaderTagBuilder::new(HeaderTagFlag::Required)
                                 .add_irs(&[MbiTagType::Framebuffer, MbiTagType::BootLoaderName]))
        .build();
    println!("{header:?}");
    file.write_all(header.as_bytes()).unwrap();
}
