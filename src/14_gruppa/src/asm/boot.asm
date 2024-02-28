global start
extern rust_main

section .multiboot_header
header_start:
    dd 0xe85250d6                ; Magic number (Multiboot 2)
    dd 0                         ; Architecture (0 for i386 protected mode)
    dd header_end - header_start ; Header length
    dd 0x100000000 - (0xe85250d6 + (header_end - header_start)) ; Checksum
    dw 0    ; End tag type
    dw 0    ; End tag flags
    dd 8    ; End tag size
header_end:

section .text
bits 32
start:
    mov esp, stack_top           ; Set stack pointer
    mov edi, ebx                 ; Move Multiboot info pointer to EDI
    call check_multiboot         ; Check Multiboot magic number
    call check_cpuid             ; Check CPUID support
    call check_long_mode         ; Check long mode availability
    call set_up_page_tables      ; Set up paging tables
    call enable_paging           ; Enable paging
    lgdt [gdt64.pointer]         ; Load 64-bit GDT
    jmp gdt64.code:long_mode     ; Jump to 64-bit mode

check_multiboot:
    cmp eax, 0x36d76289          ; Compare with magic number
    jne .no_multiboot            ; Jump if not matched
    ret
.no_multiboot:
    mov al, "0"                  ; Error code for Multiboot failure
    jmp error

check_cpuid:
    pushfd                       ; Save FLAGS
    pop eax                      ; Copy FLAGS into EAX
    mov ecx, eax                 ; Backup FLAGS
    xor eax, 1 << 21             ; Toggle ID bit
    push eax                     ; Update FLAGS
    popfd                        ; Apply updated FLAGS
    pushfd                       ; Save updated FLAGS
    pop eax                      ; Move updated FLAGS into EAX
    push ecx                     ; Restore original FLAGS
    popfd                        ; Restore FLAGS
    cmp eax, ecx                 ; Compare FLAGS for CPUID support
    je .no_cpuid                 ; Jump if CPUID not supported
    ret
.no_cpuid:
    mov al, "1"                  ; Error code for CPUID failure
    jmp error

check_long_mode:
    mov eax, 0x80000000          ; Check for extended CPU info
    cpuid                        ; Get CPU features
    cmp eax, 0x80000001          ; Check for long mode support
    jb .no_long_mode             ; Jump if not supported
    mov eax, 0x80000001          ; Request extended CPU info
    cpuid                        ; Get extended CPU features
    test edx, 1 << 29            ; Test for long mode bit
    jz .no_long_mode             ; Jump if long mode not supported
    ret
.no_long_mode:
    mov al, "2"                  ; Error code for long mode failure
    jmp error

set_up_page_tables:
    ; Initialize page table mappings
    mov eax, p4_table            ; Map P4 table
    or eax, 0b11                 ; Set present and writable flags
    mov [p4_table + 511 * 8], eax ; Self-reference P4
    mov eax, p3_table            ; Link P4 to P3 table
    or eax, 0b11                 ; Set present and writable flags
    mov [p4_table], eax          ; Link P4 to P3
    mov eax, p2_table            ; Link P3 to P2 table
    or eax, 0b11                 ; Set present and writable flags
    mov [p3_table], eax          ; Link P3 to P2
    xor ecx, ecx                 ; Reset counter for loop
.map_p2_table:
    mov eax, 0x200000            ; 2MiB page size
    mul ecx                      ; Calculate page address
    or eax, 0b10000011           ; Set present, writable, and huge page flags
    mov [p2_table + ecx * 8], eax ; Map P2 entry to page
    inc ecx                      ; Increment counter
    cmp ecx, 512                 ; Check if all entries are mapped
    jne .map_p2_table            ; Loop if not all mapped
    ret

enable_paging:
    mov eax, p4_table            ; Load P4 table address into EAX
    mov cr3, eax                 ; Set CR3 to P4 table, activating page table
    mov eax, cr4                 ; Get current CR4
    or eax, 1 << 5               ; Enable PAE
    mov cr4, eax                 ; Update CR4
    mov ecx, 0xC0000080          ; EFER MSR
    rdmsr                        ; Read EFER
    or eax, 1 << 8               ; Enable long mode
    wrmsr                        ; Write EFER
    mov eax, cr0                 ; Get current CR0
    or eax, 1 << 31              ; Enable paging
    mov cr0, eax                 ; Update CR0
    ret

error:
    ; Print error message with code
    mov dword [0xb8000], 0x4f524f45 ; 'ERRO'
    mov dword [0xb8004], 0x3a204f52 ; 'R: '
    mov byte  [0xb800a], al         ; Error code
    hlt                             ; Halt CPU

bits 64
long_mode:
    xor ax, ax                    ; Zero out segment registers
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call rust_main                ; Call Rust main function

    mov rax, 0x2f592f412f4b2f4f   ; 'OKAY'
    mov qword [0xb8000], rax
    hlt

section .bss
align 4096                       ; Align start of section to 4KB boundary for page alignment
p4_table: resb 4096              ; Reserve 4KB for P4 (level 4 page table)
p3_table: resb 4096              ; Reserve 4KB for P3 (level 3 page table)
p2_table: resb 4096              ; Reserve 4KB for P2 (level 2 page table)
stack_bottom: resb 4096 * 4      ; Reserve 16KB for stack (4 pages)
stack_top:                       ; Stack top marker (stack grows downwards)

section .rodata
gdt64:
    dq 0                         ; Null descriptor
.code: equ $ - gdt64             ; Code segment offset
    dq (1<<43) | (1<<44) | (1<<47) | (1<<53) ; Code segment descriptor
.pointer:
    dw $ - gdt64 - 1             ; GDT size
    dq gdt64                     ; GDT base address
