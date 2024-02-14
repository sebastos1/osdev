    BITS 16                  ; Tells NASM to generate 16-bit code
    ORG 0x7C00               ; Origin, BIOS loads the boot sector here

start:
    cli                      ; Disable interrupts during setup
    mov ax, cs               ; Setup segment registers
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00           ; Setup stack pointer

    ; Your initialization code here (e.g., switch to protected mode)

    ; Assuming the kernel is loaded at a specific address:
    jmp 0x100000       ; Jump to the Rust kernel entry point

    ; Boot sector padding and signature
    TIMES 510 - ($ - $$) db 0
    DW 0xAA55
