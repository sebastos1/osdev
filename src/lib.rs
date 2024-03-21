#![no_std]
#![feature(abi_x86_interrupt)]

extern crate rlibc;
extern crate alloc;

use alloc::vec::Vec;

#[macro_use]
mod vga;
mod util;
mod memory;
mod interrupts;

#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    vga::clear_screen();
    util::init();    
    memory::init(multiboot_addr);
    interrupts::init();

    let vec1: Vec<u32> = (1..=1000).collect();
    println!("vec: {:?}", vec1);
    let vec2: Vec<i32> = (-5..=5).collect();
    println!("vec: {:?}", vec2);

    println!("we made it to the loop");
    util::hlt_loop()
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! { 
    util::hlt_loop()
}