use spin::Once;
use core::arch::asm;
use super::TablePointer;
use bit_field::BitField; 
use x86_64::PrivilegeLevel; // TODO
use super::VirtualAddress;

pub static GDT: Once<Gdt> = Once::new();
pub static TSS: Once<Tss> = Once::new();

// segmentselectors are put in the segment registers (cs, ds, es, fs, gs, ss)
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    // index in GDT or LDT array (not the offset)
    // ti is ommited since we are not using LDT
    // rpl the requested privilege level
    pub const fn new(index: u16, rpl: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (rpl as u16))
    }
}

pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

pub struct Gdt {
    table: [u64; 8],
    next_free: usize,
}

impl Gdt {
    pub fn new() -> Gdt {
        Gdt {
            table: [0; 8],
            next_free: 1,
        }
    }

    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low); // Push low part.
                self.push(value_high); // Push high part.
                index
            },
        };
        SegmentSelector::new(index as u16, PrivilegeLevel::Ring0)
    }

    // Pushes a value to the GDT, panics if the GDT is full.
    fn push(&mut self, value: u64) -> usize {
        assert!(self.next_free < self.table.len(), "GDT full");
        let index = self.next_free;
        self.table[index] = value;
        self.next_free += 1;
        index
    }

    // Loads the GDT.
    pub fn load(&'static self) {

        let ptr = TablePointer {
            base: VirtualAddress(self.table.as_ptr() as u64),
            limit: (self.table.len() * core::mem::size_of::<u64>() - 1) as u16,
        };

        unsafe {
            asm!("lgdt [{}]", in(reg) &ptr, options(readonly, nostack, preserves_flags));
        }
    }
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

impl Tss {
    pub const fn new() -> Tss {
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

    pub fn descriptor(&self) -> Descriptor {
        let ptr = self as *const _ as u64;
        let mut low: u64 = 1 << 47;
        low.set_bits(16..40, ptr.get_bits(0..24)); // Base address low.
        low.set_bits(56..64, ptr.get_bits(24..32)); // Base address high part of low.
        low.set_bits(0..16, (self.iomap_base - 1) as u64); // Segment limit.
        low.set_bits(40..44, 0b1001); // Segment type (available 64-bit TSS).
        let high = ptr.get_bits(32..64); // Base address high.

        Descriptor::SystemSegment(low, high)
    }
}






