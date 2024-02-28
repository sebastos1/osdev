use spin;
use x86_64::VirtAddr;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use crate::memory::MemoryController;
use crate::gdt::{Gdt, TSS, GDT, Descriptor};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub fn init(memory_controller: &mut MemoryController) {
    use x86_64::structures::gdt::SegmentSelector;
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;
    
    let double_fault_stack = memory_controller.alloc_stack(1).expect("could not allocate double fault stack");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[crate::gdt::DOUBLE_FAULT_IST_INDEX] = VirtAddr::new(double_fault_stack.top() as u64);
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = Gdt::new();
        code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(Descriptor::tss_segment(&tss));
        gdt
    });
    gdt.load();

    unsafe {
        set_cs(code_selector); // reload code segment register
        load_tss(tss_selector); // load TSS
    }

    IDT.load();

    use core::arch::asm;
    unsafe {
        PICS.lock().initialize();
        asm!("sti", options(preserves_flags, nostack)); // enables interrupts
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX as u16);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        idt.page_fault.set_handler_fn(page_fault_handler); // test this!

        idt
    };
}

use x86_64::structures::idt::PageFaultErrorCode;

// timer is needed, or double fault happens
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop {
        x86_64::instructions::hlt(); 
    }
}

use core::sync::atomic::Ordering;
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame)
{
    crate::pit::SYSTEM_TICKS.fetch_add(1, Ordering::SeqCst);
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()); }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame)
{
    use spin::Mutex;
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = 
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8()); }
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, error: u64) -> ! {
    println!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", frame); // this isnt printed for some reason, whatevs
    panic!("{}", error);
}