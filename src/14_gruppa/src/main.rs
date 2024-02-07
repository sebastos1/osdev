#![no_std]
#![no_main]

// use crate::println;
use os::println;
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use bootloader::bootinfo::MemoryMap;
use bootloader::bootinfo::MemoryRegionType;
use os::interrupts::*;

const HZ: u32 = 100;

fn print_memory_layout(memory_map: &MemoryMap) {
    println!("Memory Regions:");
    for region in memory_map.iter() {
        let start = region.range.start_addr();
        let end = region.range.end_addr();
        let region_type = match region.region_type {
            MemoryRegionType::Usable => "Usable",
            MemoryRegionType::Reserved => "Reserved",
            // Handle other types as needed
            _ => "Other",
        };
        println!("Start: {:X}, End: {:X}, Type: {}", start, end, region_type);
    }
}

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", 123123);
    
    os::init();

    const lol: u32 = 100;
    init_pit(lol);

    // Play a beep sound
    // play_sound(10000);
    // busy_sleep(1000);
    // stop_sound();

    print_memory_layout(&boot_info.memory_map);

    loop {
        println!("playing sound of 10000");
        play_sound(5000);
        busy_sleep(1000);
        stop_sound();
        // x86_64::instructions::hlt(); // just does ("hlt" :::: "volatile")
    }
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt(); 
    }
}