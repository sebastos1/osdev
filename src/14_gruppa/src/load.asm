BITS 16                  ; Tells NASM to generate 16-bit code
ORG 0x7C00               ; Origin, BIOS loads the boot sector here

start:
    cli                      ; Disable interrupts during setup

    ; Set up the GDT
    lgdt [gdt_descriptor]

     ; Enable protected mode
    mov eax, cr0             ; Load CRO into eax
    or eax, 0x1              ; Set bit 0 
    mov cr0, eax             ; Store back into CRO

    ; Jump to 32-bit code segment
    jmp CODE_SEG:protected_mode_start

; Define GDT segment descriptors
gdt_start:
    dq 0x0000000000000000 ; Null segment
    dq 0x00009A000000FFFF ; Code segment
    dq 0x000092000000FFFF ; Data segment
gdt_end:

; GDT descriptor
gdt_descriptor:
    dw gdt_end - gdt_start - 1   ; Limit
    dd gdt_start                  ; Base

; 32-bit protected mode code segment
protected_mode_start:
    ; Set up segment registers for 32-bit mode
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Set up stack pointer
    mov esp, 0x90000

    ; Call the Rust kernel entry point
    call kernel_main

    ; Halt if the kernel returns
    cli
    hlt

; Define segment constants
CODE_SEG equ gdt_start + 0x08
DATA_SEG equ gdt_start + 0x10

; Assuming the kernel is loaded at a specific address:
kernel_main:
    jmp 0x100000       ; Jump to the Rust kernel entry point

    ; Boot sector padding and signature
    TIMES 510 - ($ - $$) db 0
    DW 0xAA55
