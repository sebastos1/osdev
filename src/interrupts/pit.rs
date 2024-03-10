use crate::util::outb;

const PIT_BASE_FREQUENCY: u32 = 1_193_182;
pub const TIMER_FREQUENCY: u32 = 100; // 10 ms per tick

// pub fn init() {
//     let divisor = PIT_BASE_FREQUENCY / TIMER_FREQUENCY;
    
//     outb(0x43, 0x36);
//     outb(0x40, (divisor & 0xFF) as u8);
//     outb(0x40, ((divisor >> 8) & 0xFF) as u8);
// }

use x86_64::instructions::port::Port;
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