ENTRY(start)
SECTIONS {
    . = 1M;
    .boot : { KEEP(*(.multiboot_header)) }
    .bss : ALIGN(4096) {}
}
