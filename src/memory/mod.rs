use spin::Once;
use spin::Mutex;
use multiboot2::{BootInformation, BootInformationHeader};

mod frame_allocator;

pub static BOOT_INFO: Once<BootInformation> = Once::new();

use frame_allocator::FrameAllocator;
pub static FRAME_ALLOCATOR: Once<Mutex<FrameAllocator>> = Once::new();

pub fn init(multiboot_addr: usize) {
    let boot_info = BOOT_INFO.call_once(||unsafe {
        BootInformation::load(multiboot_addr as *const BootInformationHeader).unwrap()
    });

    let frame_allocator = FRAME_ALLOCATOR.call_once(|| { Mutex::new(FrameAllocator::new(boot_info)) });

    // allocate some frames:
    let mut asdg = frame_allocator.lock();
    for _ in 0..100 {
        asdg.allocate();
        println!("next free: {:?}", asdg.next_free);
    }
    println!("we made it to the end of memory init")

}