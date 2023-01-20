# Blaze
A ~~probably not very~~ 🔥🚀 ***BLAZINGLY*** 🚀🔥 fast OS written in Rust 🦀🦀

## Dependencies
- Linux
- Rust nightly, aswell as the x86_64-unknown-none target.
- QEMU (for running standalone) and BOCHS (for debugging)
  - Instructions for building BOCHS can be found [here](https://wiki.osdev.org/Bochs#Compiling_Bochs_from_Source)
- Make (for building [limine](https://github.com/limine-bootloader/limine))

## How to run

### Cloning submodules
Currently, the kernel uses a [custom fork](https://github.com/calwe/x86_64/tree/master) of the [x86_64](https://github.com/rust-osdev/x86_64) crate written by phil-opp. 
This custom fork simply expands the size of the GDT, although at somepoint it would be ideal to move away from this crate entirely. This means you need to also clone the respective submodules.

### Building userspace
Running the kernel requires having a built userspace (despite userspace being disabled currently).
This can be done by entering `userspace/user_test` and running `cargo rustc -- -C relocation-model=static`.
This is because the ELF loader in its current form does 0 relocation of any symbols, so the usermode program needs to respect this.

### Running the OS
Once this is built, and submodules are pulled, you can run the OS in qemu by simply running `cargo run --bin kernel`.\
\
This command can be customised through environment variables (placed before execution `KEY=VALUE cargo run --bin kernel`)
- `RUNNER`
  - `RUNNER=bochs` - run with standard bochs
  - `RUNNER=bochs-gui` - run with the bochs gui debugger
  - `RUNNER=none` - do not run, but still build the iso
