pub mod gdt;
pub mod idt;
pub mod pit;

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
    pit::init();
    idt::init();

    // enable interrupts
    unsafe {
        crate::interrupts::idt::PICS.lock().initialize();
        core::arch::asm!("sti", options(preserves_flags, nostack));
    }
    println!("Interrupts enabled");
}