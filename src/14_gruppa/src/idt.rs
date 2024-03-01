use spin;
use x86_64::VirtAddr;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use crate::pit::SYSTEM_TICKS;
use core::sync::atomic::Ordering;
use crate::memory::MemoryController;
use crate::gdt::{Gdt, TSS, GDT, Descriptor};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::instructions::tables::load_tss;
use x86_64::registers::control::Cr2;
#[allow(deprecated)]
use x86_64::instructions::segmentation::set_cs;
use x86_64::structures::gdt::SegmentSelector;
use core::arch::asm;

// Initialize the GDT, TSS, and IDT
pub fn init(memory_controller: &mut MemoryController) {
    // Set up a stack for handling double fault exceptions
    let double_fault_stack = memory_controller.alloc_stack(1).expect("could not allocate double fault stack");

    // Initialize a TSS and assign the double fault stack
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[crate::gdt::DOUBLE_FAULT_IST_INDEX] = VirtAddr::new(double_fault_stack.top() as u64);
        tss
    });

    // Initialize GDT entries for code segment and TSS
    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = Gdt::new();
        code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(Descriptor::tss_segment(&tss));
        gdt
    });
    gdt.load();

    // Reload code segment register and load TSS
    unsafe {
        #[allow(deprecated)]
        set_cs(code_selector); // Reload code segment
        load_tss(tss_selector); // Load task state segment
    }

    // Load the IDT
    IDT.load();

    // Initialize and enable interrupts
    unsafe {
        PICS.lock().initialize(); // Initialize PICs
        asm!("sti", options(preserves_flags, nostack)); // Enable interrupts
    }
}

// Lazy static initialization of the IDT
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Set handlers for various interrupts
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe { idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX as u16); }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        idt.page_fault.set_handler_fn(page_fault_handler); // Handle page faults
        idt
    };
}

// Definitions for PIC offsets
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
// Static definition for chained PICs
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Enum for interrupt index mapping
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

// Utility methods for InterruptIndex
impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// Handlers for various interrupts
extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("\nEXCEPTION: BREAKPOINT\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _: u64) -> ! {
    panic!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); } // Halt execution on page fault
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame)
{
    SYSTEM_TICKS.fetch_add(1, Ordering::SeqCst); // Increment system ticks
    // Notify PIC about the end of interrupt
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()); }
}

extern "x86-interrupt" fn keyboard_handler(
    _stack_frame: InterruptStackFrame
) {
    // Read scancode from keyboard and add to queue
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    // Notify PIC about the end of interrupt
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}