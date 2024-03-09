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
