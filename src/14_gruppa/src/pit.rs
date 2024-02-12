use crate::println;
use core::sync::atomic::{AtomicU64, Ordering};

pub static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

const PIT_BASE_FREQUENCY: u32 = 1_193_182;
const TIMER_FREQUENCY: u32 = 100; // 10ms/tick

pub fn init() {
    use x86_64::instructions::port::Port;

    let divisor = PIT_BASE_FREQUENCY / TIMER_FREQUENCY;
    let low = (divisor & 0xFF) as u8;
    let high = ((divisor >> 8) & 0xFF) as u8;

    let mut command_port = Port::new(0x43);
    let mut data_port_channel_0 = Port::new(0x40);

    unsafe {
        command_port.write(0x36u8);
        data_port_channel_0.write(low);
        data_port_channel_0.write(high);
    }
}

pub fn play_melody() {
    let melody: [(u16, u64, u64); 1] = [
        (261, 1000, 1000) // 4th octave c note for 1 second with 1 second pause
    ];

    for &(frequency, duration, pause_duration) in melody.iter() {
        println!("playing sound of {}hz for {}ms with a {}ms pause", frequency, duration, pause_duration);
        play_note_and_pause(frequency, duration, pause_duration);
    }
}

fn play_note_and_pause(frequency: u16, duration: u64, pause_duration: u64) {
    if frequency > 0 {
        play_sound(frequency);
        busy_sleep(duration);
        stop_sound();
    }

    if pause_duration > 0 {
        busy_sleep(pause_duration);
    }
}

fn play_sound(frequency: u16) {
    use x86_64::instructions::port::Port;

    let mut speaker_control = Port::<u8>::new(0x61);
    let mut pit_control = Port::<u8>::new(0x43);
    let mut pit2_data = Port::<u8>::new(0x42);

    let divisor: u16 = (PIT_BASE_FREQUENCY / frequency as u32) as u16;

    unsafe {
        pit_control.write(0xb6); // square wave mode
        pit2_data.write((divisor & 0xff) as u8);
        pit2_data.write((divisor >> 8) as u8);

        let temp = speaker_control.read();
        speaker_control.write(temp | 0x3); // enable
    }
}

fn stop_sound() {
    use x86_64::instructions::port::Port;
    let mut speaker_control = Port::<u8>::new(0x61);

    unsafe {
        let temp = speaker_control.read();
        speaker_control.write(temp & !0x3);
    }
}

fn busy_sleep(ms: u64) {
    let target_ticks = ms / 10;

    let start_ticks = SYSTEM_TICKS.load(Ordering::SeqCst);
    while SYSTEM_TICKS.load(Ordering::SeqCst) < start_ticks + target_ticks {
        x86_64::instructions::hlt();
    }
}