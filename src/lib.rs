#![no_std]
#![feature(abi_x86_interrupt)]

extern crate rlibc;
extern crate alloc;

#[macro_use]
mod vga;
mod util;
mod music;
mod memory;
mod interrupts;

#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    vga::clear_screen();
    util::init();    
    memory::init(multiboot_addr);
    interrupts::init();

    music::play_songs();
    println!("Loop reached");
    util::hlt_loop()
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! { 
    util::hlt_loop()
}