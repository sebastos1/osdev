#![no_std]
#![no_main]

extern crate alloc;
use alloc::{vec::Vec}; // use this for heap allocation
use bootloader::BootInfo;
use core::panic::PanicInfo;
use os::{println, hlt_loop};

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    use os::memory;
    use os::allocator;
    use x86_64::VirtAddr;
    use os::memory::BootInfoFrameAllocator;

    println!("Hello World! {} x {} = {}", 2, 4, 2*4);
    os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let n = 1000;
    let mut asdf = Vec::new();
    for i in 0..n {
        asdf.push(i);
    }
    println!("big vec: {:?}", asdf);

    // music
    loop {
        use os::sound::play_melody;
        play_melody();
    }

    // println!("loop reached :)");
    // hlt_loop();
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    hlt_loop();
}