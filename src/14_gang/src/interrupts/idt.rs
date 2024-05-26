use core::arch::asm;
use super::handlers;
use super::pic::PIC_OFFSET;
use lazy_static::lazy_static;
use core::ops::{Index, IndexMut};
use super::{TablePointer, Address};
use super::gdt::{SegmentSelector, DOUBLE_FAULT_IST_INDEX, GDT};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    DivideError,
    DoubleFault = 8,
    GeneralProtectionFault = 13,
    PageFault = 14,
    Timer = PIC_OFFSET,
    Keyboard,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
struct IdtEntry {
    fn_pointer_low: u16,
    cs: SegmentSelector, // u16
    ist: u8,
    flags: u8,
    fn_pointer_middle: u16,
    fn_pointer_high: u32,
    reserved: u32,
}

impl IdtEntry {
    fn new() -> Self {
        IdtEntry {
            flags: 0b1110, // interrupt gate
            ..unsafe { core::mem::zeroed() }
        }
    }

    fn set_handler(&mut self, handler: extern "x86-interrupt" fn()) -> &mut Self {
        let address = handler as u64;
        self.fn_pointer_low = address as u16;
        self.fn_pointer_middle = (address >> 16) as u16;
        self.fn_pointer_high = (address >> 32) as u32;
        self.cs = GDT.selectors.code;
        self.flags |= 0b10000000;
        self
    }

    fn with_ist_index(&mut self, index: usize) {
        self.ist = index as u8;
    }
}

#[derive(Clone, Debug)]
#[repr(C, align(16))]
struct Idt([IdtEntry; 256]);

impl Idt {
    fn new() -> Self {
        Idt([IdtEntry::new(); 256])
    }

    fn load(&self) {
        let pointer = TablePointer {
            base: Address(self.0.as_ptr() as u64),
            limit: (core::mem::size_of::<Self>() - 1) as u16,
        };
        unsafe {
            asm!(
                "lidt [{}]",
                in(reg) &pointer,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}

impl Index<InterruptIndex> for Idt {
    type Output = IdtEntry;
    fn index(&self, index: InterruptIndex) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<InterruptIndex> for Idt {
    fn index_mut(&mut self, index: InterruptIndex) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

pub fn init() {
    lazy_static! {
        static ref IDT: Idt = {
            let mut idt = Idt::new();
            idt[InterruptIndex::Timer].set_handler(handlers::timer_interrupt);
            idt[InterruptIndex::DoubleFault].set_handler(handlers::double_fault).with_ist_index(DOUBLE_FAULT_IST_INDEX);
            idt[InterruptIndex::Keyboard].set_handler(handlers::keyboard_interrupt);
            idt[InterruptIndex::DivideError].set_handler(handlers::divide_error);
            idt[InterruptIndex::GeneralProtectionFault].set_handler(handlers::general_protection_fault);
            idt[InterruptIndex::PageFault].set_handler(handlers::page_fault);
            idt
        };
    }

    IDT.load();
}