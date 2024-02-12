#![no_std]
#![no_main]

extern crate alloc;
use os::println;
use alloc::{vec::Vec}; // use this for heap allocation
use bootloader::BootInfo;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    use os::memory;
    use os::allocator;
    use x86_64::VirtAddr;
    use os::memory::MemoryFrameAllocator;

    println!("Hello World! {} x {} = {}", 2, 4, 2*4);
    os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::initialize_offset_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe { MemoryFrameAllocator::new(&boot_info.memory_map) };

    allocator::init(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let n = 1000;
    let mut asdf = Vec::new();
    for i in 0..n {
        asdf.push(i);
    }
    println!("big vec: {:?}", asdf);

    // music
    loop {
        use os::pit::play_melody;
        play_melody();
    }

    // println!("loop reached :)");
    // loop {
    //     x86_64::instructions::hlt();
    // }
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}