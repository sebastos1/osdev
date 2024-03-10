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
    crate::interrupts::init();

    println!("we made it to the loop");
    loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); }}
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! { 
    loop {}
} 