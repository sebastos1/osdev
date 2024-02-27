use core::mem::size_of;
use lazy_static::lazy_static;
// use x86_64::instructions::segmentation::Segment;
// use x86_64::{structures::{tss::TaskStateSegment, gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector}}, VirtAddr};

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct TssEntry {
    reserved: u32,
    rsp: [u64; 3],
    reserved2: u64,
    ist: [u64; 7],
    reserved3: u64,
    reserved4: u16,
    iomap_base: u16,
}




#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}
impl GdtEntry {
    fn new(base: u32, limit: u32, access: u8, granularity: u8) -> Self {
        GdtEntry {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            granularity: (limit >> 16) & 0x0F | (granularity & 0xF0),
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }
} // explain this code, very straight forward how gdt entires work.



#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u64, // In 64-bit mode, the base is 64 bits.
}


lazy_static! {
    static mut ref GDT: [GdtEntry; 3] = [
        GdtEntry::new(0, 0, 0, 0), // Null descriptor (doesn't hurt)
        GdtEntry::new(0, 0xFFFFFFFF, 0x9A, 0xCF), // Code segment
        GdtEntry::new(0, 0xFFFFFFFF, 0x92, 0xCF), // Data segment
    ];
}

extern "C" {
    fn load_gdt(gdtr: *const Gdtr);
}

fn init_gdt() {
    let gdtr = Gdtr {
        limit: (size_of::<GdtEntry>() * GDT.len() - 1) as u16,
        base: unsafe { GDT.as_ptr() as u64 },
    };

    unsafe {
        load_gdt(&gdtr);
    }
}

// Assembly function to load GDTR
global_asm!(
    ".globl load_gdt",
    "load_gdt:",
    "lgdt [rdi]",
    "ret"
);





// struct Selectors {
//     code: SegmentSelector,
//     data: SegmentSelector,
//     tss: SegmentSelector,
// }


// pub const DOUBLE_FAULT_IST_INDEX: u16 = 1;

// lazy_static! {
//     static ref TSS: TaskStateSegment = {
//         let mut tss = TaskStateSegment::new();
//         static mut STACK: [u8; 4096 * 5] = [0; 4096 * 5];
//         let stack_top = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK) }) + (4096 * 5) * size_of::<u8>() as u64;
//         tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize - 1] = stack_top;
//         tss
//     };
// }

// lazy_static! {
//     static ref GDT: (GlobalDescriptorTable, Selectors) = {
//         let mut gdt = GlobalDescriptorTable::new();
//         let code = gdt.add_entry(Descriptor::kernel_code_segment());
//         let data = gdt.add_entry(Descriptor::kernel_data_segment());
//         let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));
//         (gdt, Selectors { code, data, tss })
//     };
// }

// pub fn init() {
//     GDT.0.load();
//     unsafe {
//         x86_64::instructions::segmentation::CS::set_reg(GDT.1.code);
//         x86_64::instructions::segmentation::DS::set_reg(GDT.1.data);
//         x86_64::instructions::tables::load_tss(GDT.1.tss);
//     }
