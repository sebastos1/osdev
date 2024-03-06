#![no_std]

extern crate rlibc;

mod vga;

#[no_mangle]
pub extern fn rust_main() {
    vga::clear_screen();  
    print!("Hello World {}", 123);
    
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! { 
    loop {}
}