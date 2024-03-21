grub_cfg      := "grub.cfg"
linker_script := "linker.ld"
target        := "x86_64-os"
iso           := "build/os-x86_64.iso"
kernel        := "build/kernel-x86_64.bin"
rust_os       := "target/x86_64-os/debug/libos.a"
asm           := "src/asm/boot.asm"
asm_obj       := "build/asm/boot.o"

default: run

@build_kernel:
    cargo build -Zbuild-std --target {{target}}.json
    mkdir -p build/asm
    nasm -felf64 {{asm}} -o {{asm_obj}}
    ld -n --gc-sections -T {{linker_script}} -o {{kernel}} {{asm_obj}} {{rust_os}} 2>/dev/null

@build: build_kernel
    mkdir -p build/iso/boot/grub
    cp {{kernel}} build/iso/boot/kernel.bin
    cp {{grub_cfg}} build/iso/boot/grub
    grub-mkrescue -o {{iso}} build/iso 2> /dev/null
    rm -r build/iso

@run: build
    echo "Running..."
    qemu-system-x86_64 -cdrom {{iso}} -audiodev pa,id=speaker -machine pcspk-audiodev=speaker -m 2048M

@clean:
    rm -r build