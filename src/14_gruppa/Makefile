arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso 
target ?= $(arch)-os
rust_os := target/$(target)/debug/libos.a

linker_script := linker.ld
grub_cfg := grub.cfg
assembly_source_files := $(wildcard src/asm/*.asm)
assembly_object_files := $(patsubst src/asm/%.asm, \
	build/asm/%.o, $(assembly_source_files))

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@echo "Running..."
	@qemu-system-x86_64 -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)

kernel:
	@cargo build -Zbuild-std --target $(target).json 

build/asm/%.o: src/asm/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@