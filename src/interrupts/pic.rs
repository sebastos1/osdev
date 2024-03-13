use crate::util::{inb, outb};

pub const PIC_OFFSET: u8 = 32;
pub static PICS: spin::Mutex<Pics> = spin::Mutex::new(Pics::new(PIC_OFFSET));

struct Pic {
    offset: u8,
    command_port: u16,
    data_port: u16,
}

impl Pic {
    const fn new(offset: u8, command_port: u16, data_port: u16) -> Pic {
        Pic { offset, command_port, data_port }
    }

    fn handles_interrupt(&self, interrupt_index: u8) -> bool {
        self.offset <= interrupt_index && interrupt_index < self.offset + 8 // each PIC handles 8 interrupts
    }
}

// chained pics
pub struct Pics {
    primary: Pic,
    secondary: Pic,
}

impl Pics {
    pub const fn new(offset: u8) -> Pics {
        Pics {
            primary: Pic::new(offset, 0x20, 0x21),
            secondary: Pic::new(offset + 8, 0xA0, 0xA1),
        }
    }

    pub fn send_eoi(&mut self, interrupt_index: super::idt::InterruptIndex) {
        if self.secondary.handles_interrupt(interrupt_index as u8) {
            outb(self.secondary.command_port, 0x20); // secondary
        }
        outb(self.primary.command_port, 0x20); // primary
    }
}

pub fn init() {
    let pics = PICS.lock();
    let (primary, secondary) = (&pics.primary, &pics.secondary);
    let wait = || outb(0x80, 0); // delay: PIC needs time to initialize

    // original masks
    let primary_mask = inb(primary.data_port);
    let secondary_mask = inb(secondary.data_port);

    // 3 byte init sequence
    let cmd_init = 0x11;
    outb(primary.command_port, cmd_init);
    outb(secondary.command_port, cmd_init);
    wait();

    // base offsets
    outb(primary.data_port, primary.offset);
    outb(secondary.data_port, secondary.offset);
    wait();

    // chaining
    outb(primary.data_port, 4); // Tell PIC1 that there is a PIC2 at IRQ2 (0000 0100)
    outb(secondary.data_port, 2); // Tell PIC2 its cascade identity (0000 0010)
    wait();

    // 8086 mode
    outb(primary.data_port, 0x01);
    outb(secondary.data_port, 0x01);
    wait();

    // restore masks
    outb(primary.data_port, primary_mask);
    outb(secondary.data_port, secondary_mask);
}