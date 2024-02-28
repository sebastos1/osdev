#![no_std]
#![feature(asm_const)]
#![feature(allocator_api)]
#![feature(abi_x86_interrupt)]

extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate multiboot2;

#[macro_use]
mod vga;
mod pit;
mod gdt;
mod idt;
mod util;
mod memory;

use linked_list_allocator::LockedHeap;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

use multiboot2::BootInformation;
use spin::Once;
static BOOT_INFO: Once<BootInformation> = Once::new();

#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    util::init();
    vga::clear_screen();
    println!("Hello World! {}", 5*5);
    
    gdt::init();
    

    let boot_info = unsafe {
        multiboot2::BootInformation::load(multiboot_addr as *const multiboot2::BootInformationHeader).unwrap()
    };
    BOOT_INFO.call_once(|| boot_info);
    let mut memory_controller = memory::init();
    unsafe { HEAP_ALLOCATOR.lock().init(crate::memory::HEAP_START as *mut u8, crate::memory::HEAP_START + crate::memory::HEAP_SIZE); }

    idt::init(&mut memory_controller);

    // use alloc::vec::Vec;
    // let vec: Vec<i32> = (1..=1000).collect();
    // println!("{:?}", vec);

    // pit::init();

    // breakpoint
    // x86_64::instructions::interrupts::int3();
    
    // double fault
    // unsafe { *(0xdeadbeaf as *mut u64) = 42; };

    println!("It did not crash!");
    loop{
        x86_64::instructions::hlt(); 
    }
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop { x86_64::instructions::hlt(); }
}