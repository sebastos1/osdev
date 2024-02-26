#![no_std]
#![feature(allocator_api)]

extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate multiboot2;

#[macro_use]
mod vga;
mod util;
mod memory;

use linked_list_allocator::LockedHeap;
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    vga::clear_screen();
    println!("Hello World! {}", 5*5);
    
    util::set_bits_init();
    memory::init(multiboot_information_address);

    unsafe {
        HEAP_ALLOCATOR.lock().init(crate::memory::HEAP_START as *mut u8, crate::memory::HEAP_START + crate::memory::HEAP_SIZE);
    }

    use alloc::vec::Vec;
    let vec: Vec<i32> = (1..=1000).collect();
    println!("{:?}", vec);
    
    println!("It did not crash!");
    loop{ x86_64::instructions::hlt(); }
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { 
    loop { x86_64::instructions::hlt(); }
}