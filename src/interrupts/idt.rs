use spin::Once;
use core::arch::asm;
use lazy_static::lazy_static;
use super::gdt::SegmentSelector;
use core::sync::atomic::Ordering;
use core::sync::atomic::AtomicU64;
use super::pic::{PIC_OFFSET, PICS};
use super::{TablePointer, VirtualAddress};
use x86_64::structures::idt::InterruptStackFrame;

pub static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);
pub static IDT: Once<Idt> = Once::new();

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    DoubleFault = 8,
    Timer = PIC_OFFSET,
    Keyboard,
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
    fn set_handler(&mut self, handler: extern "x86-interrupt" fn(InterruptStackFrame)) -> &mut Self {
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
    let idt = IDT.call_once(|| {
        let mut idt = Idt::default();
        idt[InterruptIndex::Timer as usize].set_handler(timer_interrupt_handler);
        idt[InterruptIndex::DoubleFault as usize].set_handler(double_fault_handler).with_ist_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        idt[InterruptIndex::Keyboard as usize].set_handler(keyboard_interrupt_handler);
        idt
    });

    idt.load();
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame) {
    print!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", frame);
    loop {};
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame)
{
    // print!(".");
    SYSTEM_TICKS.fetch_add(1, Ordering::SeqCst); // Increment system ticks
    PICS.lock().send_eoi(InterruptIndex::Timer as u8);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_: InterruptStackFrame) {
    use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use crate::interrupts::norwegian::No105Key;
    use spin::Mutex;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<No105Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(ScancodeSet1::new(), No105Key, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let scancode: u8 = crate::util::inb(0x60);

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    if character == '\n' {
                        println!();
                    } else if (0x20..=0x7e).contains(&(character as u32)) {
                        print!("{}", character);
                    }
                },
                DecodedKey::RawKey(_) => {}
            }
        }
    }

    PICS.lock().send_eoi(InterruptIndex::Keyboard as u8);
}