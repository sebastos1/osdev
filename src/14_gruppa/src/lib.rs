#![no_std]
#![feature(abi_x86_interrupt)]

pub mod vga;
pub mod interrupts;
pub mod gdt;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    interrupts::init_pit(100); // 100 hz
}

use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
pub fn print_memory_layout(memory_map: &MemoryMap) {

    println!("\nMemory Regions:");
    for region in memory_map.iter() {
        let start = region.range.start_addr();
        let end = region.range.end_addr();
        let region_type = match region.region_type {
            MemoryRegionType::Usable => "Usable",
            MemoryRegionType::Reserved => "Reserved",
            _ => "Other",
        };
        println!("Start: {:X}, End: {:X}, Type: {}", start, end, region_type);
    }
}