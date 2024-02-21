#![no_std]

extern crate rlibc;

#[macro_use]
mod vga;

#[no_mangle]
pub extern fn rust_main() {
    vga::clear_screen();
    println!("Hello World{}", 3 + 2);

    loop{}
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}