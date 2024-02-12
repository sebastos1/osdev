use x86_64::{VirtAddr, structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB}};
use linked_list_allocator::LockedHeap;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;
    let heap_range = Page::range_inclusive(
        Page::containing_address(heap_start),
        Page::containing_address(heap_end),
    );

    for page in heap_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            mapper.map_to(page, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_allocator)?.flush();
        }
    }

    unsafe { HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); }

    Ok(())
}
