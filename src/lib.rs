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

    tests();

    println!("we made it to the loop");
    util::hlt_loop();
}

#[allow(dead_code)]
fn tests() {
    // divide error - 1/0
    // unsafe { core::arch::asm!("mov eax, 1", "mov ebx, 0", "div ebx", options(nostack)); }

    // double fault
    // unsafe { *(0xdeadbeef as *mut u8) = 42; };
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! { 
    util::hlt_loop();
} 