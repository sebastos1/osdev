global start
extern rust_main

section .text
mb_start:
    dd 0xe85250d6                 ; Magic number (Multiboot 2)
    dd 0                          ; Architecture (0 for i386 protected mode)
    dd mb_end - mb_start          ; Header length
    dd 0x100000000 - (0xe85250d6 + (mb_end - mb_start)) ; Checksum
    dw 0                          ; End tag type
    dw 0                          ; End tag flags
    dd 8                          ; End tag size
mb_end:

bits 32
start:
    mov esp, stack_top            ; Set stack pointer
    mov edi, ebx                  ; Move Multiboot info pointer to EDI
    call paging                   ; Set up paging tables
    lgdt [gdt.pointer]            ; Load 64-bit GDT
    jmp gdt.code:long_mode        ; Jump to 64-bit mode and Rust code

paging:                           ; Initialize page table mappings
    mov eax, p4_table
    or eax, 0b11                  ; present + writable
    mov [p4_table + 511 * 8], eax ; Self-reference P4
    mov eax, p3_table
    or eax, 0b11                  ; present + writable
    mov [p4_table], eax           ; Link P4 to P3
    mov eax, p2_table
    or eax, 0b11                  ; present + writable
    mov [p3_table], eax           ; Link P3 to P2
    xor ecx, ecx                  ; i=0
.map_p2:
    mov eax, 0x200000             ; 2MiB pages
    mul ecx                       ; 2MiB * i for addr
    or eax, 0b10000011            ; present + writable + huge page
    mov [p2_table + ecx * 8], eax ; Map entry to page
    inc ecx                       ; i++
    cmp ecx, 512                  ; Check if mapped, or
    jne .map_p2                   ; loop if not

    ; When done mapping:
    mov eax, p4_table
    mov cr3, eax                  ; Put P4 in CR3
    mov eax, cr4                  ; CR4
    or eax, 1 << 5                ; Set PAE bit
    mov cr4, eax                  ; Save CR4
    mov ecx, 0xC0000080           ; EFER MSR address
    rdmsr                         ; Read EFER
    or eax, 1 << 8                ; Set LME bit (long mode)
    wrmsr                         ; Write EFER
    mov eax, cr0                  ; Get current CR0
    or eax, 1 << 31               ; Set PG bit (paging)
    mov cr0, eax                  ; Update CR0
    ret

gdt:
    dq 0                          ; Null seg
.code: equ $ - gdt                ; Offset, for long mode jump
    dq 0x20980000000000           ; Code seg
.pointer:                         ; Pointer for lgdt
    dw $ - gdt - 1                ; Size as 16-bit
    dq gdt                        ; Base address

bits 64
long_mode:
    xor ax, ax                    ; Zero out segment registers
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call rust_main                ; Go to rust code
    hlt

section .bss
align 4096                        ; Page alignment
p4_table: resb 4096               ; Reserve 4KiB for P4
p3_table: resb 4096               ; Reserve 4KiB for P3
p2_table: resb 4096               ; Reserve 4KiB for P2
stack_bottom: resb 4096 * 4       ; Reserve 4 pages for stack
stack_top:                        ; Stack top marker