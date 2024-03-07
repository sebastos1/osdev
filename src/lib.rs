#![no_std]

extern crate rlibc;

mod vga;

static TEST: u32 = 0xDEADBEEF;

#[no_mangle]
pub extern fn rust_main() {
    vga::clear_screen();

    let address = &TEST as *const u32 as usize;
    println!("Hello world! Address: 0x{:X}", address);
    
    loop {}
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! { 
    loop {}
}