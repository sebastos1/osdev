use spin;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
// use crate::memory::MemoryController;
use core::sync::atomic::Ordering;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub fn init() { // memory_controller: &mut MemoryController
    // let double_fault_stack = memory_controller.alloc_stack(1).expect("could not allocate double fault stack");

    IDT.load();
}

// x86 crate has a list of idt entries
// https://docs.rs/x86_64/latest/x86_64/structures/idt/struct.InterruptDescriptorTable.html
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe { idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX); }
        // idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
        // idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);

        // this doesn't work yet, only the breakpoint handler works

        idt
    };
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_usize(self) -> usize {
        self as u8 as usize
    }

    fn as_u8(self) -> u8 {
        self as u8
    }
}

extern "x86-interrupt" fn timer_handler(_: InterruptStackFrame) {
    println!("huh");
    crate::pit::SYSTEM_TICKS.fetch_add(1, Ordering::SeqCst);
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8); }
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame)
{
    println!("hello");

    use spin::Mutex;
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8()); }
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", frame);
}