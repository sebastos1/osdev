#![no_std]
#![feature(asm_const)]
#![feature(allocator_api)]
#![feature(abi_x86_interrupt)]

extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate bit_field;
extern crate multiboot2;

#[macro_use]
mod vga;
mod pit;
mod gdt;
mod idt;
mod util;
mod memory;
mod console;
pub mod task;

use spin::Once;
use crate::task::keyboard;
use multiboot2::BootInformation;
use linked_list_allocator::LockedHeap;
use crate::task::{Task, executor::Executor};

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

static BOOT_INFO: Once<BootInformation> = Once::new();

#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    util::init();
    vga::clear_screen();
    println!("Hello world! {}", 123);
    
    let boot_info = unsafe {
        multiboot2::BootInformation::load(multiboot_addr as *const multiboot2::BootInformationHeader).unwrap()
    };
    BOOT_INFO.call_once(|| boot_info);
    let mut memory_controller = memory::init();
    unsafe { HEAP_ALLOCATOR.lock().init(crate::memory::HEAP_START as *mut u8, crate::memory::HEAP_START + crate::memory::HEAP_SIZE); }

    pit::init(); // sets tick speed to 100hz
    idt::init(&mut memory_controller);

    console::init();

    tests();

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    // executor.spawn(Task::new(example_task())); 
    executor.run();
}

fn tests() {
    // use alloc::vec::Vec;
    // let big_vec: Vec<i32> = (1..=1000).collect();
    // println!("big_vec {:?}", big_vec);

    // breakpoint
    // x86_64::instructions::interrupts::int3();
    
    // double fault
    // println!("Invoking double fault now!");
    // unsafe { *(0xdeadbeef as *mut u64) = 42; };

    // page fault
    // let ptr = 0xdeadbeaf as *mut u8;
    // unsafe { *ptr = 42; }
}

#[allow(unused)]
async fn async_number() -> u32 {
    42
}

#[allow(unused)]
async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! { 
    vga::VGA_WRITER.lock().set_text_color(crate::vga::VgaColor::Red);
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}