use core::mem::size_of;
use lazy_static::lazy_static;
use core::arch::global_asm;
use core::arch::asm;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    fn new(base: u32, limit: u32, access: u8, granularity: u8) -> Self {
        GdtEntry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: ((limit >> 16) & 0x0F) as u8 | (granularity & 0xF0),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct TssEntry {
    part1: GdtEntry, // Lower 32 bits of base address, segment limit, access rights, and flags
    base_upper: u32, // Upper 32 bits of base address
    reserved: u32,   // Typically 0
}


impl TssEntry {
    fn new(base: u64, limit: u32) -> Self {
        TssEntry {
            part1: GdtEntry::new(base as u32, limit, 0x89, 0x00),
            base_upper: (base >> 32) as u32,
            reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u64,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved1: u32,
    rsp: [u64; 3],
    reserved2: u64,
    ist: [u64; 7],
    reserved3: u64,
    reserved4: u16,
    iomap_base: u16,
}

static DOUBLE_FAULT_STACK: [u8; 4096] = [0; 4096];
pub static DOUBLE_FAULT_IST_INDEX: u16 = 1;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment {
            reserved1: 0,
            rsp: [0; 3],
            reserved2: 0,
            ist: [0; 7], // Initialize all IST entries to 0
            reserved3: 0,
            reserved4: 0,
            iomap_base: 0,
        };

        // Set IST1 (used for double faults) to the top of the DOUBLE_FAULT_STACK
        // Remember: Stack grows downwards, so we point to the end of the array
        tss.ist[0] = &DOUBLE_FAULT_STACK as *const _ as u64 + DOUBLE_FAULT_STACK.len() as u64;

        tss
    };
}

lazy_static! {
    static ref GDT: ([GdtEntry; 3], TssEntry) = {
        let tss_base = &*TSS as *const _ as u64;
        let tss_limit = size_of::<TaskStateSegment>() as u32 - 1;
        let tss_descriptor = TssEntry::new(tss_base, tss_limit);

        let code = GdtEntry::new(0, 0xFFFF_FFFF, 0x9A, 0xA0); // 0x9A = Present, executable, read/write, accessed
        let data = GdtEntry::new(0, 0xFFFF_FFFF, 0x92, 0xA0); // 0x92 = Present, read/write, accessed

        ([GdtEntry::new(0, 0, 0, 0), code, data], tss_descriptor) // Simplified to only include TSS
    };
}

extern "C" {
    fn load_gdt(gdtr: *const Gdtr);
}

global_asm!(
    ".globl load_gdt",
    "load_gdt:",
    "lgdt [rdi]",
    "ret"
);

// when we update the GDT, we want to add new entries to the GDT to ensure it doesn't reference the old entries
pub fn init() {
    let gdtr = Gdtr {
        limit: (size_of::<([GdtEntry; 3], TssEntry)>() - 1) as u16,
        base: &*GDT as *const _ as u64,
    };

    unsafe {
        load_gdt(&gdtr);

        // Use general-purpose registers to load segment selectors
        asm!(
            "push {code_segment}",
            "lea rax, [rip + 1f]",
            "push rax",
            "retfq",
            "1:",
            "mov ax, {data_segment}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            code_segment = const 0x08, // Assuming code segment is at GDT index 1 (0x08)
            data_segment = const 0x10, // Assuming data segment is at GDT index 2 (0x10)
            options(nostack)
        );

        // Load the TSS using a general-purpose register
        asm!(
            "mov ax, {tss_selector}",
            "ltr ax",
            tss_selector = const 0x18, // Assuming TSS is at GDT index 3 (0x18)
            options(nomem)
        );
    }
}

