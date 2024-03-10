use spin;
use spin::Once;
use core::arch::asm;
use bit_field::BitField;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use super::gdt::SegmentSelector;
use core::sync::atomic::Ordering;
use core::sync::atomic::AtomicU64;
use super::{TablePointer, VirtualAddress};

pub static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);
pub static IDT: Once<Idt> = Once::new();

#[no_mangle]
extern "C" fn timer_interrupt_handler() {
    print!(".");
    SYSTEM_TICKS.fetch_add(1, Ordering::SeqCst);
    unsafe { 
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8);
        asm!("sti", options(nomem, nostack, preserves_flags)); // wtf ?
    }
}

pub const PIC_OFFSET: u8 = 32;
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_OFFSET, (PIC_OFFSET + 8) as u8) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_OFFSET,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct IdtEntry {
    fn_pointer_low: u16,
    cs: SegmentSelector, // u16
    ist: u8,
    flags: u8,
    fn_pointer_middle: u16,
    fn_pointer_high: u32,
    reserved: u32,
}

impl Default for IdtEntry {
    fn default() -> Self {
        IdtEntry {
            fn_pointer_low: 0,
            fn_pointer_middle: 0,
            fn_pointer_high: 0,
            cs: SegmentSelector(0),
            ist: 0, // assume no ist, use with_ist_index to set
            flags: 0b1110, // interrupt gate
            reserved: 0,
        }
    }
}

impl IdtEntry {
    fn set_handler(&mut self, handler: extern "C" fn()) -> &mut Self {
        let address = handler as u64;
        self.fn_pointer_low = address as u16;
        self.fn_pointer_middle = (address >> 16) as u16;
        self.fn_pointer_high = (address >> 32) as u32;
        
        let mut segment: u16;
        unsafe {
            asm!("mov {0:x}, cs", out(reg) segment, options(nomem, nostack, preserves_flags));
        }
        self.cs = SegmentSelector(segment);
        self.flags |= 0b10000000;
        self
    }

    #[allow(dead_code)]
    fn with_ist_index(&mut self, index: u8) {
        self.ist = index;
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct Idt(pub [IdtEntry; 256]);

impl Default for Idt {
    fn default() -> Self {
        Idt([IdtEntry::default(); 256])
    }
}

impl Idt {
    fn load(&self) {
        let pointer = TablePointer {
            base: VirtualAddress(self as *const _ as u64),
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

impl core::ops::Index<usize> for Idt {
    type Output = IdtEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl core::ops::IndexMut<usize> for Idt {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

pub fn init() {
    unsafe { PICS.lock().initialize(); }
    let idt = IDT.call_once(|| {
        let mut idt = Idt::default();
        idt[InterruptIndex::Timer as usize].set_handler(timer_interrupt_handler);
        // idt.double_fault.set_handler(double_fault_handler).with_ist_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        idt
    });

    idt.load();
}