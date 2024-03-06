global start
extern rust_main

section .multiboot_header
h_start:
    dd 0xe85250d6                 ; Magic number (Multiboot 2)
    dd 0                          ; Architecture (0 for i386 protected mode)
    dd h_end - h_start            ; Header length
    dd 0x100000000 - (0xe85250d6 + (h_end - h_start)) ; Checksum
    dw 0                          ; End tag type
    dw 0                          ; End tag flags
    dd 8                          ; End tag size
h_end:

section .text
bits 32
start:
    mov esp, stack_top            ; Set stack pointer
    mov edi, ebx                  ; Move Multiboot info pointer to EDI
    call paging_setup             ; Set up paging tables
    call paging_start             ; Enable paging
    lgdt [gdt64.pointer]          ; Load 64-bit GDT
    jmp gdt64.code:long_mode      ; Jump to 64-bit mode and Rust code

paging_setup:                     ; Initialize page table mappings
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
.map_p2_table:
    mov eax, 0x200000             ; 2MiB page size
    mul ecx                       ; Calculate page address
    or eax, 0b10000011            ; present + writable + huge page
    mov [p2_table + ecx * 8], eax ; Map P2 entry to page
    inc ecx                       ; i++
    cmp ecx, 512                  ; Check if all entries are mapped
    jne .map_p2_table             ; Loop if not all mapped
    ret

paging_start:
    mov eax, p4_table
    mov cr3, eax                  ; Set CR3 to P4 table, activating page table
    mov eax, cr4                  ; Get current CR4
    or eax, 1 << 5                ; Set PAE bit
    mov cr4, eax                  ; Update CR4
    mov ecx, 0xC0000080           ; EFER MSR address
    rdmsr                         ; Read EFER
    or eax, 1 << 8                ; Set LME bit (long mode)
    wrmsr                         ; Write EFER
    mov eax, cr0                  ; Get current CR0
    or eax, 1 << 31               ; Set PG bit (paging)
    mov cr0, eax                  ; Update CR0
    ret

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
p4_table: resb 4096               ; Reserve 4KB for P4
p3_table: resb 4096               ; Reserve 4KB for P3
p2_table: resb 4096               ; Reserve 4KB for P2
stack_bottom: resb 4096 * 4       ; Reserve 4 pages for stack
stack_top:                        ; Stack top marker

section .rodata
gdt64:
    dq 0                          ; Null descriptor
.code: equ $ - gdt64              ; Offset
    dq 0x20980000000000           ; Code segment descriptor
.pointer:
    dw $ - gdt64 - 1              ; Size
    dq gdt64                      ; Base address
