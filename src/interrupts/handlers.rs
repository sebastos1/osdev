use spin::Mutex;
use super::SYSTEM_TICKS;
use super::pic::PICS;
use lazy_static::lazy_static;
use super::norwegian::No105Key;
use core::sync::atomic::Ordering;
use super::idt::InterruptIndex;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1};

pub extern "x86-interrupt" fn divide_error() {
    print!("\nEXCEPTION: DIVIDE ERROR\n");
    crate::util::hlt_loop();
}

pub extern "x86-interrupt" fn double_fault() {
    print!("\nEXCEPTION: DOUBLE FAULT\n");
    crate::util::hlt_loop();
}

pub extern "x86-interrupt" fn timer_interrupt() {
    // print!(".");
    SYSTEM_TICKS.fetch_add(1, Ordering::SeqCst);
    PICS.lock().send_eoi(InterruptIndex::Timer);
}

pub extern "x86-interrupt" fn keyboard_interrupt() {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<No105Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(ScancodeSet1::new(), No105Key, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let scancode: u8 = crate::util::inb(0x60);

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(DecodedKey::Unicode(character)) = keyboard.process_keyevent(key_event) {
            match character {
                '\n' => println!(),
                _ if (0x20..=0x7e).contains(&(character as u32)) => print!("{}", character),
                _ => {}
            }
        }
    }
    
    PICS.lock().send_eoi(InterruptIndex::Keyboard);
}