#![no_std]

extern crate rlibc;
extern crate alloc;
extern crate multiboot2;

#[macro_use]
mod vga;
mod memory;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize) {
    use crate::memory::FrameAllocator;

    vga::clear_screen();
    println!("Hello World{}", "!");

    let boot_info = unsafe { 
        multiboot2::BootInformation::load(
            multiboot_information_address as *const multiboot2::BootInformationHeader
        ).unwrap() 
    };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let elf_sections = boot_info.elf_sections().expect("elf sections required");
    let kernel_start = elf_sections.clone().map(|s| s.start_address()).min().unwrap();
    let kernel_end = elf_sections.map(|s| s.start_address() + s.size()).max().unwrap();
    let multiboot_start = multiboot_information_address;
    let multiboot_end = multiboot_start + (boot_info.total_size() as usize);
 
    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize, 
        kernel_end as usize, 
        multiboot_start,
        multiboot_end, 
        memory_map_tag.memory_areas()
    );

    for i in 0.. {
        if let None = frame_allocator.allocate_frame() {
            println!("allocated {} frames", i);
            break;
        }
    }

    loop{ x86_64::instructions::hlt(); }
}

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { x86_64::instructions::hlt(); }
}
