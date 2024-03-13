use core::arch::asm;
use core::mem::size_of;
use bit_field::BitField;
use lazy_static::lazy_static;
use super::{TablePointer, VirtualAddress};

pub const DOUBLE_FAULT_IST_INDEX: usize = 0;

lazy_static! {
    static ref TSS: Tss = {
        let mut tss = Tss::new();
        // stack for the double fault handler
        tss.ist[DOUBLE_FAULT_IST_INDEX] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_end = unsafe { STACK.as_ptr() as u64 + STACK_SIZE as u64 };
            VirtualAddress(stack_end)
        };
        tss
    };

    static ref GDT: (Gdt, Selectors) = {
        let mut gdt = Gdt::new();
        let code = gdt.add_entry(Descriptor::UserSegment(0x20980000000000));
        let tss = gdt.add_entry(TSS.descriptor());
        (gdt, Selectors {code, tss})
    };
}

enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(4))]
struct Tss {
    reserved_1: u32,
    rsp: [VirtualAddress; 3],
    reserved_2: u64,
    ist: [VirtualAddress; 7],
    reserved_3: u64,
    reserved_4: u16,
    iomap_base: u16,
}

impl Tss {
    fn new() -> Tss {
        Tss {
            reserved_1: 0,
            rsp: [VirtualAddress(0); 3],
            reserved_2: 0,
            ist: [VirtualAddress(0); 7],
            reserved_3: 0,
            reserved_4: 0,
            iomap_base: size_of::<Tss>() as u16,
        }
    }

    fn descriptor(&self) -> Descriptor {
        let ptr = self as *const _ as u64;
        let mut low: u64 = 1 << 47;
        low.set_bits(16..40, ptr.get_bits(0..24)); // low
        low.set_bits(56..64, ptr.get_bits(24..32)); // middle
        low.set_bits(0..16, (self.iomap_base - 1) as u64); // limit
        low.set_bits(40..44, 0b1001); // segment type
        let high = ptr.get_bits(32..64); // high

        Descriptor::SystemSegment(low, high)
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

struct Gdt {
    table: [u64; 8],
    next: usize,
}

impl Gdt {
    fn new() -> Self {
        Gdt {
            table: [0; 8],
            next: 1,
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
            base: VirtualAddress(self.table.as_ptr() as u64),
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
    let (gdt, selectors) = &*GDT;

    gdt.load();

    unsafe {
        asm!( // set cs
            "push {sel}",
            "lea {tmp}, [1f + rip]",
            "push {tmp}",
            "retfq",
            "1:",
            sel = in(reg) u64::from(selectors.code.0),
            tmp = lateout(reg) _,
            options(preserves_flags),
        );
        asm!( // load tss
            "ltr {0:x}",
            in(reg) selectors.tss.0,
            options(preserves_flags),
        );
    }
}