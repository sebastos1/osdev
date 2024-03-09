#![no_std]
#![feature(abi_x86_interrupt)]

extern crate rlibc;

#[macro_use]
mod vga;
mod util;
mod interrupts;

#[no_mangle]
pub extern fn rust_main() {
    vga::clear_screen();
    println!("Hello world! {}", 123);
    
    util::init();
    crate::interrupts::pit::init();
    // crate::interrupts::idt::init();
    
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! { 
    loop {}
}