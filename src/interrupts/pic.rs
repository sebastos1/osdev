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

    pub fn send_eoi(&mut self, interrupt_index: u8) {
        if self.secondary.handles_interrupt(interrupt_index) {
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

    // Tell each PIC that we're going to send it a three-byte initialization sequence on its data port.
    let cmd_init = 0x11;
    outb(primary.command_port, cmd_init); // Command sent to begin PIC initialization.
    outb(secondary.command_port, cmd_init);
    wait();

    // Byte 1: Set up our base offsets.
    outb(primary.data_port, primary.offset);
    outb(secondary.data_port, secondary.offset);
    wait();

    // Byte 2: Configure chaining between PIC1 and PIC2.
    outb(primary.data_port, 4); // Tell PIC1 that there is a PIC2 at IRQ2 (0000 0100)
    outb(secondary.data_port, 2); // Tell PIC2 its cascade identity (0000 0010)
    wait();

    // Byte 3: Set our mode.
    outb(primary.data_port, 0x01); // 8086 mode
    outb(secondary.data_port, 0x01);
    wait();

    // Restore our saved masks.
    outb(primary.data_port, primary_mask);
    outb(secondary.data_port, secondary_mask);
}