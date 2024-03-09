use crate::util::outb;

const PIT_BASE_FREQUENCY: u32 = 1_193_182;
pub const TIMER_FREQUENCY: u32 = 100; // 10 ms per tick

pub fn init() {
    let divisor = PIT_BASE_FREQUENCY / TIMER_FREQUENCY;
    
    outb(0x43, 0x36);
    outb(0x40, (divisor & 0xFF) as u8);
    outb(0x40, ((divisor >> 8) & 0xFF) as u8);
}