use spin;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use core::sync::atomic::Ordering;
use crate::memory::MemoryController;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub fn init(memory_controller: &mut MemoryController) {
    let double_fault_stack = memory_controller.alloc_stack(1).expect("could not allocate double fault stack");
    IDT.load();
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);


        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error: u64) -> ! {
    println!("\nEXCEPTION: DOUBLE FAULT\n{:#?}", frame);
    loop {}
}