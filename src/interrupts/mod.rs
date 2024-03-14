use core::arch::asm;

mod gdt;
mod pic;
mod idt;
mod handlers;
mod norwegian;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct VirtualAddress(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct PhysicalAddress(pub u64);

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct TablePointer {
    pub limit: u16,
    pub base: VirtualAddress,
}

pub fn init() {
    gdt::init();
    pic::init();
    idt::init();

    // enable interrupts
    unsafe { asm!("sti", options(preserves_flags, nostack)); }
}