use crate::util::outb;
use super::SYSTEM_TICKS;
use core::sync::atomic::Ordering;

pub const PIT_BASE_FREQUENCY: u32 = 1_193_182;
pub const TIMER_FREQUENCY: u32 = 100; // 10 ms per tick

pub fn init() {
    let divisor = PIT_BASE_FREQUENCY / TIMER_FREQUENCY;

    outb(0x43, 0x36);
    outb(0x40, (divisor & 0xFF) as u8);
    outb(0x40, ((divisor >> 8) & 0xFF) as u8);
}

pub fn sleep_busy(milliseconds: u32) {
    let start_tick = SYSTEM_TICKS.load(Ordering::SeqCst);
    let ticks_to_wait = milliseconds * TIMER_FREQUENCY / 1000;
    let mut elapsed_ticks = 0;

    while elapsed_ticks < ticks_to_wait {
        while SYSTEM_TICKS.load(Ordering::SeqCst) == start_tick + elapsed_ticks as u64 {
            // chillin'
        }
        elapsed_ticks += 1;
    }
}