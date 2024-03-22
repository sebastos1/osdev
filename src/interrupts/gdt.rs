use core::arch::asm;
use bit_field::BitField;
use lazy_static::lazy_static;
use core::mem::{size_of, zeroed};
use super::{TablePointer, Address};

pub const DOUBLE_FAULT_IST_INDEX: usize = 0;

lazy_static! {
    static ref TSS: Tss = {
        let mut tss = Tss::new();
        // stack for the double fault handler
        tss.ist[DOUBLE_FAULT_IST_INDEX] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_end = unsafe { STACK.as_ptr() as u64 + STACK_SIZE as u64 };
            Address(stack_end)
        };
        tss
    };

    pub static ref GDT: Gdt = {
        let mut gdt = Gdt::new();
        let code = gdt.add_entry(Descriptor::UserSegment(0x20980000000000));
        let tss = gdt.add_entry(TSS.descriptor());
        gdt.selectors = Selectors {code, tss};
        gdt
    };
}

enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct Tss {
    _reserved_1: u32,
    _rsp: [Address; 3],
    _reserved_2: u64,
    ist: [Address; 7],
    _reserved_3: u64,
    _reserved_4: u16,
    iomap_base: u16,
}

impl Tss {
    fn new() -> Tss {
        Tss {
            iomap_base: size_of::<Tss>() as u16,
            ..unsafe { zeroed() }
        }
    }

    fn descriptor(&self) -> Descriptor {
        let ptr = self as *const _ as u64;
        let mut low = 1 << 47;
        low.set_bits(0..16, (self.iomap_base - 1) as u64)
            .set_bits(16..40, ptr.get_bits(0..24))
            .set_bits(40..44, 0b1001)
            .set_bits(56..64, ptr.get_bits(24..32));

        Descriptor::SystemSegment(low, ptr.get_bits(32..64))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    pub const fn new(index: u16) -> SegmentSelector {
        SegmentSelector(index << 3)
    }
}

pub struct Selectors {
    pub code: SegmentSelector,
    pub tss: SegmentSelector,
}

pub struct Gdt {
    table: [u64; 8],
    next: usize,
    pub selectors: Selectors,
}

impl Gdt {
    fn new() -> Self {
        Gdt {
            next: 1,
            ..unsafe { zeroed() }
        }
    }

    fn push(&mut self, value: u64) -> usize {
        let index = self.next;
        self.table[index] = value;
        self.next += 1;
        index
    }

    fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low);
                self.push(value_high);
                index
            },
        };
        SegmentSelector::new(index as u16)
    }

    fn load(&self) {
        let pointer = TablePointer {
            base: Address(self.table.as_ptr() as u64),
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };
        unsafe {
            asm!(
                "lgdt [{}]",
                in(reg) &pointer,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}

pub fn init() {
    let gdt = &GDT;

    gdt.load();

    // cs and tss
    unsafe {
        asm!(
            "push {sel}",
            "lea {tmp}, [1f + rip]",
            "push {tmp}",
            "retfq",
            "1:",
            sel = in(reg) u64::from(gdt.selectors.code.0),
            tmp = lateout(reg) _,
            options(preserves_flags),
        );
        asm!(
            "ltr {0:x}",
            in(reg) gdt.selectors.tss.0,
            options(preserves_flags),
        );
    }
}