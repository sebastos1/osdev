ENTRY(start)
SECTIONS {
    . = 1M;
    .boot : { KEEP(*(.multiboot_header)) }
    .text ALIGN(4096) : { *(.text .text.*) }
    .rodata ALIGN(4096) : { *(.rodata .rodata.*) }
    .data : ALIGN(4096) { *(.data .data.*) }
    .bss : ALIGN(4096) { *(.bss .bss.*) *(COMMON) }
}