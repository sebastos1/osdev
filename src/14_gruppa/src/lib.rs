#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod idt;
pub mod gdt;
pub mod pit;
pub mod vga;
pub mod memory;
pub mod allocator;

// Function to initialize the various subsystems
pub fn init() {
    gdt::init();
    idt::init();
    pit::init();
    unsafe { idt::PICS.lock().initialize() }; // Adjusted to match the PIC_CHAIN name
    x86_64::instructions::interrupts::enable();
}