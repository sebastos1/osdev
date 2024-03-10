use spin::Once;
use core::arch::asm;
use bit_field::BitField; 
use lazy_static::lazy_static;
use super::{TablePointer, VirtualAddress};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: Tss = {
        let mut tss = Tss::default();
        tss.ist[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_end = unsafe {
                let stack_start = core::ptr::addr_of!(STACK) as *const _ as u64;
                stack_start + STACK_SIZE as u64
            };
            VirtualAddress(stack_end)
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (Gdt, Selectors) = {
        let mut gdt = Gdt::default();
        let cs = gdt.add_entry(Descriptor::UserSegment(0x20980000000000));
        let ts = gdt.add_entry(TSS.descriptor());
        (
            gdt,
            Selectors {
                cs,
                ts,
            },
        )
    };
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
            sel = in(reg) u64::from(selectors.cs.0),
            tmp = lateout(reg) _,
            options(preserves_flags),
        );
        asm!( // load tss
            "ltr {0:x}",
            in(reg) selectors.ts.0,
            options(preserves_flags),
        );
    }

}

pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(4))]
pub struct Tss {
    reserved_1: u32,
    pub rsp: [VirtualAddress; 3],
    reserved_2: u64,
    pub ist: [VirtualAddress; 7],
    reserved_3: u64,
    reserved_4: u16,
    pub iomap_base: u16,
}

impl Default for Tss {
    fn default() -> Tss {
        Tss {
            reserved_1: 0,
            rsp: [VirtualAddress(0); 3],
            reserved_2: 0,
            ist: [VirtualAddress(0); 7],
            reserved_3: 0,
            reserved_4: 0,
            iomap_base: core::mem::size_of::<Tss>() as u16,
        }
    }
}

impl Tss {
    pub fn descriptor(&self) -> Descriptor {
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    // index in GDT (not the offset)
    // ti is ommited since we are not using LDT (always 0)
    // rpl the requested privilege level
    pub const fn new(index: u16, rpl: u16) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl))
    }
}

pub struct Selectors {
    pub cs: SegmentSelector,
    pub ts: SegmentSelector,
}

pub struct Gdt {
    table: [u64; 8],
    next_free: usize,
}

impl Default for Gdt {
    fn default() -> Self {
        Gdt {
            table: [0; 8],
            next_free: 1,
        }
    }
}

impl Gdt {
    fn push(&mut self, value: u64) -> usize {
        let index = self.next_free;
        self.table[index] = value;
        self.next_free += 1;
        index
    }

    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low);
                self.push(value_high);
                index
            },
        };
        SegmentSelector::new(index as u16, 0)
    }

    pub fn load(&'static self) {
        let ptr = TablePointer {
            base: VirtualAddress(self.table.as_ptr() as u64),
            limit: (self.table.len() * core::mem::size_of::<u64>() - 1) as u16,
        };

        unsafe {
            asm!(
                "lgdt [{}]",
                in(reg) &ptr,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}
