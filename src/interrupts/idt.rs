use spin;
use core::arch::asm;
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use super::{TablePointer, VirtualAddress};
use super::gdt::{Gdt, GDT, Tss, TSS, Descriptor, SegmentSelector};

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::default();
        // SET THE HANDLER FUNCTIONS HERE
        idt.breakpoint.set_handler(breakpoint_handler);
        idt
    };
}

/*
TODO wrap ExceptionStackFrame!

extern "x86-interrupt" fn handler(stack_frame: ExceptionStackFrame) {…}
extern "x86-interrupt" fn handler_with_err_code(stack_frame: ExceptionStackFrame, error_code: u64) {…}
*/

extern "x86-interrupt" fn breakpoint_handler() {
    println!("\nEXCEPTION: BREAKPOINT\n");
}

// Represents the 4 non-offset bytes of an IDT entry.
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct EntryOptions {
    cs: SegmentSelector,
    bits: u16,
}

impl Default for EntryOptions {
    fn default() -> Self {
        EntryOptions {
            cs: SegmentSelector(0),
            bits: 0b1110_0000_0000,
        }
    }
}

// we always have the same cs, so we dont inlcude that here
#[derive(Clone, Copy)]
#[repr(C)]
pub struct IdtEntry {
    pointer_low: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl Default for IdtEntry {
    fn default() -> Self {
        IdtEntry {
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::default(),
            reserved: 0,
        }
    }
}

impl IdtEntry {
    fn set_handler(&mut self, function: extern "x86-interrupt" fn()) {
        let address = function as u64;
        self.pointer_low = address as u16;
        self.pointer_middle = (address >> 16) as u16;
        self.pointer_high = (address >> 32) as u32;
        // options?
    }
}

#[repr(C)]
pub struct Idt {
    divide_by_0: IdtEntry,
    debug: IdtEntry,
    nmi: IdtEntry, // Non-maskable interrupt
    breakpoint: IdtEntry,
    overflow: IdtEntry,
    bound_range_exceeded: IdtEntry,
    invalid_opcode: IdtEntry, 
    device_not_available: IdtEntry,
    double_fault: IdtEntry,
    coprocessor_segment_overrun: IdtEntry,
    invalid_tss: IdtEntry,
    segment_not_present: IdtEntry,
    stack_segment_fault: IdtEntry,
    general_protection_fault: IdtEntry, 
    page_fault: IdtEntry,
    reserved_1: IdtEntry,
    x87_floating_point: IdtEntry,
    alignment_check: IdtEntry,
    machine_check: IdtEntry,
    simd_floating_point: IdtEntry,
    virtualization: IdtEntry,
    cp_protection_exception: IdtEntry,
    reserved_2: [IdtEntry; 6],
    hv_injection_exception: IdtEntry, 
    vmm_communication_exception: IdtEntry,
    security_exception: IdtEntry,
    reserved_3: IdtEntry,
    interrupts: [IdtEntry; 256 - 32], // hardware interrupts and such
}

impl Default for Idt {
    fn default() -> Self {
        Idt {
            divide_by_0: IdtEntry::default(),
            debug: IdtEntry::default(),
            nmi: IdtEntry::default(),
            breakpoint: IdtEntry::default(),
            overflow: IdtEntry::default(),
            bound_range_exceeded: IdtEntry::default(),
            invalid_opcode: IdtEntry::default(),
            device_not_available: IdtEntry::default(),
            double_fault: IdtEntry::default(),
            coprocessor_segment_overrun: IdtEntry::default(),
            invalid_tss: IdtEntry::default(),
            segment_not_present: IdtEntry::default(),
            stack_segment_fault: IdtEntry::default(),
            general_protection_fault: IdtEntry::default(),
            page_fault: IdtEntry::default(),
            reserved_1: IdtEntry::default(),
            x87_floating_point: IdtEntry::default(),
            alignment_check: IdtEntry::default(),
            machine_check: IdtEntry::default(),
            simd_floating_point: IdtEntry::default(),
            virtualization: IdtEntry::default(),
            cp_protection_exception: IdtEntry::default(),
            reserved_2: [IdtEntry::default(); 6],
            hv_injection_exception: IdtEntry::default(),
            vmm_communication_exception: IdtEntry::default(),
            security_exception: IdtEntry::default(),
            reserved_3: IdtEntry::default(),
            interrupts: [IdtEntry::default(); 256 - 32],
        }
    }
}

impl Idt {
    fn load(&self) {
        let pointer = TablePointer {
            base: VirtualAddress(self as *const _ as u64),
            limit: (core::mem::size_of::<Self>() - 1) as u16,
        };
        unsafe {
            asm!("lidt [{}]", in(reg) &pointer, options(readonly, nostack, preserves_flags));
        }
    }
}

pub fn init() {
    let tss = TSS.call_once(|| {
        let mut tss = Tss::default();
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = Gdt::default();
        code_selector = gdt.add_entry(Descriptor::UserSegment(0x20980000000000)); // kernel code segment, same as from the asm code
        tss_selector = gdt.add_entry(tss.descriptor());
        gdt
    });
    gdt.load();

    unsafe {
        asm!(
            "push {sel}",
            "lea {tmp}, [1f + rip]",
            "push {tmp}",
            "retfq",
            "1:",
            sel = in(reg) u64::from(code_selector.0),
            tmp = lateout(reg) _,
            options(preserves_flags),
        );
        asm!(
            "ltr {0:x}",
            in(reg) tss_selector.0,
            options(preserves_flags),
        );
    }

    IDT.load();

    // // enable interrupts
    // unsafe {
    //     // PICS.lock().initialize();
    //     asm!("sti", options(preserves_flags, nostack));
    // }
}