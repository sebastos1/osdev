use core::sync::atomic::AtomicU64;
use x86_64::instructions::port::Port;

pub static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

pub const PIT_BASE_FREQUENCY: u32 = 1_193_182;
pub const TIMER_FREQUENCY: u32 = 100; // 10ms/tick

pub fn init() {
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

/*
// Enum to represent different types of sound events
enum SoundEvent {
    PlaySound(u16),
    StopSound,
}

// DelayEvent struct for non-blocking delays, now using SoundEvent
struct DelayEvent {
    start_tick: u64,
    duration: u64,
    event: SoundEvent,
}

lazy_static! {
    static ref DELAY_EVENTS: Mutex<Vec<DelayEvent>> = Mutex::new(Vec::new());
}

pub fn play_melody() {
    let melody: [(u16, u64, u64); 3] = [
        (261, 1000, 1000), // 4th octave c note for 1 second with 1 second pause
        (400, 500, 1500), // Example note and durations
        (100, 1500, 500), // Example note and durations
    ];

    let mut start_time = SYSTEM_TICKS.load(Ordering::SeqCst);
    for &(frequency, duration, pause_duration) in &melody {
        println!("Scheduling sound of {}Hz for {}ms with a {}ms pause", frequency, duration, pause_duration);
        if frequency > 0 {
            add_delay_event(DelayEvent {
                start_tick: start_time,
                duration,
                event: SoundEvent::PlaySound(frequency),
            });
        }

        start_time += duration;

        if pause_duration > 0 {
            add_delay_event(DelayEvent {
                start_tick: start_time,
                duration: pause_duration,
                event: SoundEvent::StopSound,
            });

            start_time += pause_duration;
        }
    }
}

fn add_delay_event(event: DelayEvent) {
    let mut events = DELAY_EVENTS.lock();
    events.push(event);
}

// Call this function periodically, e.g., from your main loop or an interrupt handler
pub fn check_delay_events() {
    let mut events = DELAY_EVENTS.lock();
    let current_tick = SYSTEM_TICKS.load(Ordering::SeqCst);

    events.retain(|event| {
        if current_tick >= event.start_tick {
            match event.event {
                SoundEvent::PlaySound(frequency) => play_sound(frequency),
                SoundEvent::StopSound => stop_sound(),
            }
            false // Event handled, remove it
        } else {
            true // Event not yet handled, keep it
        }
    });
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
*/