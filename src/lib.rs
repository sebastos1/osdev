#![no_std]
#![feature(abi_x86_interrupt)]

extern crate rlibc;
// extern crate alloc;

#[macro_use]
mod vga;
mod util;
mod memory;
mod interrupts;

// use alloc::vec::Vec;

#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    vga::clear_screen();
    util::init();    
    memory::init(multiboot_addr);
    interrupts::init();

    // big vec with the numbers from 1 to 1000
    // let vec: Vec<u32> = (1..=5).collect();
    // println!("vec: {:?}", vec);

    println!("we made it to the loop");
    util::hlt_loop()
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_: &PanicInfo) -> ! { 
    util::hlt_loop()
}