use core::mem::size_of;
use lazy_static::lazy_static;
use x86_64::instructions::segmentation::Segment;
use x86_64::{structures::{tss::TaskStateSegment, gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector}}, VirtAddr};

struct Selectors {
    code: SegmentSelector,
    data: SegmentSelector,
    tss: SegmentSelector,
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        static mut STACK: [u8; 4096 * 5] = [0; 4096 * 5];
        let stack_top = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK) }) + (4096 * 5) * size_of::<u8>() as u64;
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_top;
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new(); // null is added
        let code = gdt.add_entry(Descriptor::kernel_code_segment());
        let data = gdt.add_entry(Descriptor::kernel_data_segment());
        let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code, data, tss })
    };
}

pub fn init() {
    GDT.0.load();
    unsafe {
        x86_64::instructions::segmentation::CS::set_reg(GDT.1.code);
        x86_64::instructions::segmentation::DS::set_reg(GDT.1.data);
        x86_64::instructions::tables::load_tss(GDT.1.tss);
    }
}
