use spin::Once;
use x86_64::VirtAddr;
use x86_64::PrivilegeLevel;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::tss::TaskStateSegment;

// Lazy-initialized global instances for TSS and GDT.
pub static TSS: Once<TaskStateSegment> = Once::new();
pub static GDT: Once<Gdt> = Once::new();

// Index for the Double Fault Interrupt Stack Table (IST).
pub static DOUBLE_FAULT_IST_INDEX: usize = 0;

// Defines flags for GDT descriptors using bitflags for easy manipulation.
bitflags! {
    struct DescriptorFlags: u64 {
        const CONFORMING        = 1 << 42;
        const EXECUTABLE        = 1 << 43;
        const USER_SEGMENT      = 1 << 44;
        const PRESENT           = 1 << 47;
        const LONG_MODE         = 1 << 53;
    }
}

// Represents the Global Descriptor Table (GDT).
pub struct Gdt {
    table: [u64; 8], // Stores up to 8 entries.
    next_free: usize, // Tracks the next free slot in the table.
}

impl Gdt {
    // Creates a new, empty GDT.
    pub fn new() -> Gdt {
        Gdt {
            table: [0; 8],
            next_free: 1, // Start from 1 since 0 is not used.
        }
    }

    // Adds a descriptor to the GDT and returns its segment selector.
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
        use x86_64::instructions::tables::{lgdt, DescriptorTablePointer};
        let ptr = DescriptorTablePointer {
            base: VirtAddr::new(self.table.as_ptr() as u64),
            limit: (self.table.len() * core::mem::size_of::<u64>() - 1) as u16,
        };
        unsafe { lgdt(&ptr) };
    }
}

// Represents a GDT descriptor.
pub enum Descriptor {
    UserSegment(u64), // For code/data segments.
    SystemSegment(u64, u64), // For system segments like TSS.
}

impl Descriptor {
    // Returns a descriptor for a kernel code segment.
    pub fn kernel_code_segment() -> Descriptor {
        let flags = DescriptorFlags::USER_SEGMENT | DescriptorFlags::PRESENT | DescriptorFlags::EXECUTABLE | DescriptorFlags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }

    // Returns a descriptor for the TSS segment.
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        use bit_field::BitField;

        let ptr = tss as *const _ as u64;
        let mut low = DescriptorFlags::PRESENT.bits();
        low.set_bits(16..40, ptr.get_bits(0..24)); // Base address low.
        low.set_bits(56..64, ptr.get_bits(24..32)); // Base address high part of low.
        low.set_bits(0..16, (core::mem::size_of::<TaskStateSegment>() - 1) as u64); // Segment limit.
        low.set_bits(40..44, 0b1001); // Segment type (available 64-bit TSS).

        let high = ptr.get_bits(32..64); // Base address high.

        Descriptor::SystemSegment(low, high)
    }
}
