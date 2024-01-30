#![no_std]
#![no_main]

// use crate::println;
use os::println;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    
    os::init();

    loop {
        x86_64::instructions::hlt(); // just does ("hlt" :::: "volatile")
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt(); 
    }
}