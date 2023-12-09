triple := "x86_64-blaze"
profile := "debug"

ovmf_code := "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd"
ovmf_vars := "ovmf/OVMF_VARS.fd"

qemu_args := "-m 256M -debugcon stdio -no-reboot"
qemu_debug := "-S -s"

build: build-bootstrap build-kernel link-kernel prep-iso build-iso
run: build run-qemu
run-debug: build run-qemu-debug

build-bootstrap: (step "Building Bootstrap...")
    @mkdir target -p
    nasm -f elf64 -o target/boot kernel/src/boot.asm

build-kernel: (step "Building Kernel...")
    cargo build --lib -Z build-std=core --target=x86_64-blaze.json

link-kernel: (step "Linking Kernel...")
    ld -o target/{{triple}}/{{profile}}/kernel target/boot target/{{triple}}/{{profile}}/libkernel.a -Tkernel/link.ld

prep-iso: (step "Creating ISO Directory...")
    mkdir -p target/iso/boot/grub
    cp target/{{triple}}/{{profile}}/kernel target/iso/boot/
    cp grub/grub.cfg target/iso/boot/grub/

build-iso: (step "Building ISO...")
    grub-mkrescue -o target/blaze.iso target/iso

run-qemu: (step "Running in QEMU")
    qemu-system-x86_64 -cdrom target/blaze.iso {{qemu_args}} \
        -drive if=pflash,format=raw,readonly=on,file={{ovmf_code}} \
        -drive if=pflash,format=raw,file={{ovmf_vars}}

run-qemu-debug: (step "Running in QEMU (GDB Stub)")
    qemu-system-x86_64 -cdrom target/blaze.iso {{qemu_args}} {{qemu_debug}} \
        -drive if=pflash,format=raw,readonly=on,file={{ovmf_code}} \
        -drive if=pflash,format=raw,file={{ovmf_vars}}

run-bochs: build (step "Running in Bochs")
    bochs -q

new-header:
    cargo run --bin mbheader
    cp mbheader/multiboot2_header.bin kernel/src/

debug:
    gdb -s target/iso/boot/kernel -ex "target remote localhost:1234"

clean:
    cargo clean

step step:
    @echo -e "\033[1;31m{{step}}\033[0m"
