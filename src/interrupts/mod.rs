use core::arch::asm;
use core::sync::atomic::AtomicU32;

pub mod gdt;
pub mod pic;
pub mod pit;
pub mod idt;
pub mod handlers;
pub mod norwegian;

pub static SYSTEM_TICKS: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Address(pub u64);

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct TablePointer {
    pub limit: u16,
    pub base: Address,
}

pub fn init() {
    gdt::init();
    pic::init();
    pit::init();
    idt::init();

    // enable interrupts
    unsafe { asm!("sti", options(preserves_flags, nostack)); }
}