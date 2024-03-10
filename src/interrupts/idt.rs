use spin;
use core::arch::asm;
use bit_field::BitField;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use core::sync::atomic::Ordering;
use core::sync::atomic::AtomicU64;
use super::{TablePointer, VirtualAddress};
use super::gdt::SegmentSelector;

pub static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::default();
        idt.interrupts[InterruptIndex::Timer as usize].set_handler(timer_interrupt_handler);
        // idt.double_fault.set_handler(double_fault_handler).with_ist_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        idt
    };
}

#[derive(Debug)]
#[repr(C)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

extern "x86-interrupt" fn timer_interrupt_handler(_: ExceptionStackFrame) {
    SYSTEM_TICKS.fetch_add(1, Ordering::SeqCst);
    println!(".");
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8); }
}

pub const PIC_OFFSET: u8 = 32;
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_OFFSET, (PIC_OFFSET + 8) as u8) });

// Enum for interrupt index mapping
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_OFFSET,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct IdtEntry {
    pointer_low: u16,
    cs: SegmentSelector,
    bits: u16,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl Default for IdtEntry {
    fn default() -> Self {
        IdtEntry {
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            cs: SegmentSelector(0),
            bits: 0b1110_0000_0000, // Default to a 64-bit Interrupt Gate
            reserved: 0,
        }
    }
}
use super::gdt::GDT;

impl IdtEntry {
    fn set_handler(&mut self, function: extern "x86-interrupt" fn(ExceptionStackFrame)) -> &mut Self {
        let address = function as u64;
        self.pointer_low = address as u16;
        self.pointer_middle = (address >> 16) as u16;
        self.pointer_high = (address >> 32) as u32;
        
        // let mut segment: u16;
        // unsafe {
        //     asm!("mov {0:x}, cs", out(reg) segment, options(nomem, nostack, preserves_flags));
        // }
        // self.cs = SegmentSelector(segment);
        self.cs = GDT.1.cs;
        self.bits.set_bit(15, true);
        self
    }

    fn with_ist_index(&mut self, index: u16) {
        self.bits.set_bits(0..3, index + 1);
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
#[repr(align(16))]
pub struct Idt {
    unused_1: [IdtEntry; 3],
    breakpoint: IdtEntry,
    unused_2: [IdtEntry; 3],
    double_fault: IdtEntry,
    unused_3: [IdtEntry; 23],
    interrupts: [IdtEntry; 256 - 32], // hardware interrupts and such
}

impl Default for Idt {
    fn default() -> Self {
        Idt {
            unused_1: [IdtEntry::default(); 3],
            breakpoint: IdtEntry::default(),
            unused_2: [IdtEntry::default(); 3],
            double_fault: IdtEntry::default(),
            unused_3: [IdtEntry::default(); 23],
            interrupts: [IdtEntry::default(); 256 - 32],
        }
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

pub fn init() {
    IDT.load();

    // enable interrupts
    unsafe {
        PICS.lock().initialize();
        asm!("sti", options(preserves_flags, nostack));
    }
}