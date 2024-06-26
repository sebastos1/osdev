use core::arch::asm;

pub fn init() {
    enable_nxe_bit();
    enable_write_protect_bit();
}

fn enable_nxe_bit() {
    let efer_msr: u64 = 0xC000_0080; // EFER MSR number
    let mut efer: u64;
    unsafe { asm!("rdmsr", in("ecx") efer_msr, out("eax") efer, out("edx") _, options(nostack)); } // Read EFER
    efer |= 1 << 11; // Set NXE bit
    unsafe { asm!("wrmsr", in("ecx") efer_msr, in("eax") efer, in("edx") 0, options(nostack)); } // Write EFER
}

fn enable_write_protect_bit() {
    let mut cr0: u64;
    unsafe { asm!("mov {}, cr0", out(reg) cr0, options(nostack)); } // Read CR0
    cr0 |= 1 << 16; // Set Write Protect bit
    unsafe { asm!("mov cr0, {}", in(reg) cr0, options(nostack)); } // Write CR0
}

pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nostack, preserves_flags),
        );
    }
}

pub fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nostack, preserves_flags),
        );
    }
    value
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem, preserves_flags));
        }
    }
}

pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}