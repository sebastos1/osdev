use spin;
use crate::gdt;
use crate::print;
use crate::println;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use core::sync::atomic::Ordering;

use core::sync::atomic::AtomicU64;

static TICKS: AtomicU64 = AtomicU64::new(0);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub fn init_idt() {
    IDT.load();
}


// The PIT frequency
const PIT_BASE_FREQUENCY: u32 = 1_193_182;
// Desired frequency for the timer interrupts
const TIMER_FREQUENCY: u32 = 100; // for example, 100 Hz

pub fn init_pit(frequency: u32) {
    let divisor = PIT_BASE_FREQUENCY / frequency;
    let low = (divisor & 0xFF) as u8;
    let high = ((divisor >> 8) & 0xFF) as u8;

    let mut command_port = Port::new(0x43);
    let mut data_port_channel_0 = Port::new(0x40); // Channel 0 data port

    unsafe {
        command_port.write(0x36u8); // Command byte for Channel 0
        data_port_channel_0.write(low); // Divisor low byte
        data_port_channel_0.write(high); // Divisor high byte
    }
}

pub fn play_sound(frequency: u16) {
    let mut speaker_control = Port::<u8>::new(0x61);
    let mut pit_control = Port::<u8>::new(0x43);
    let mut pit2_data = Port::<u8>::new(0x42);

    let pit_freq: u32 = 1_193_182; // Frequency of the Programmable Interval Timer
    let divisor: u16 = (pit_freq / u32::from(frequency)) as u16;

    unsafe {
        // Set the PIT to square wave mode on channel 2
        pit_control.write(0xb6);
        // Set the frequency divisor
        pit2_data.write((divisor & 0xff) as u8);
        pit2_data.write((divisor >> 8) as u8);

        // Enable the speaker
        let temp = speaker_control.read();
        speaker_control.write(temp | 0x3);
    }
}

pub fn stop_sound() {
    let mut speaker_control = Port::<u8>::new(0x61);

    unsafe {
        // Disable the speaker
        let temp = speaker_control.read();
        speaker_control.write(temp & !0x3);
    }
}

pub fn busy_sleep(ms: u64) {
    let target_ticks = (ms * (TIMER_FREQUENCY as u64)) / 1000;
    let start_ticks = TICKS.load(Ordering::SeqCst);
    while TICKS.load(Ordering::SeqCst) < start_ticks + target_ticks {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
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
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    TICKS.fetch_add(1, Ordering::SeqCst);
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame)
{
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1,
                HandleControl::Ignore)
            );
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

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}