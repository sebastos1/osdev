#![no_std]
#![feature(abi_x86_interrupt)]

pub mod vga;
pub mod gdt;
pub mod sound;
pub mod memory;
pub mod allocator;
pub mod interrupts;

extern crate alloc;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    sound::init_pit();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}