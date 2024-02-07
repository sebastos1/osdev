use x86_64::VirtAddr;
use core::ptr::addr_of;
use lazy_static::lazy_static;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{SegmentSelector, GlobalDescriptorTable, Descriptor};
use x86_64::registers::segmentation::DS;


pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { addr_of!(STACK) });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // the NULL descriptor is implicitly created when the GDT is created
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment()); // text descriptor
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment()); // data descriptor
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS)); // task state segment
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};
    
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}