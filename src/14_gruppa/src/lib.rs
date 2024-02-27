#![no_std]
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
mod util;
mod memory;
mod interrupts;

use linked_list_allocator::LockedHeap;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    util::init();
    vga::clear_screen();
    println!("Hello World! {}", 5*5);
    
    gdt::init();

    pit::init();
    memory::init(multiboot_addr);
    // let mut memory_controller = memory::init(multiboot_addr);
    // interrupts::init(); // &mut memory_controller
    // x86_64::instructions::interrupts::int3();


    // unsafe {
    //     HEAP_ALLOCATOR.lock().init(memory::HEAP_START as *mut u8, memory::HEAP_START + memory::HEAP_SIZE);
    // }
    
    // double fault
    // unsafe { *(0xdeadbeaf as *mut u64) = 42; };

    // use alloc::vec::Vec;
    // let vec: Vec<i32> = (1..=1000).collect();
    // println!("{:?}", vec);
    
    println!("It did not crash!");

    // use core::sync::atomic;
    let mut i = 0;
    loop{
        // println!("Tick: {:?}", pit::SYSTEM_TICKS.load(atomic::Ordering::SeqCst));
        // println!("Tick {}", i);
        // i += 1; 
        x86_64::instructions::hlt(); 
    }
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop { x86_64::instructions::hlt(); }
}