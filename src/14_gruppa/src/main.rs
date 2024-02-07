#![no_std]
#![no_main]

// use crate::println;
use os::println;
use bootloader::BootInfo;
use core::panic::PanicInfo;
use os::print_memory_layout;


use os::interrupts::*;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    println!("Hello World! {} x {} = {}", 2, 4, 2*4);
    os::init();

    print_memory_layout(&boot_info.memory_map);

    loop {
        println!("playing sound of 5000 hz");
        play_sound(5000);
        busy_sleep(1000);
        stop_sound();
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt(); 
    }
}