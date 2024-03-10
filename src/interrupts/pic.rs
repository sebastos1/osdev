use crate::util::{inb, outb};

pub const PIC_OFFSET: u8 = 32;
pub static PICS: spin::Mutex<Pics> = spin::Mutex::new(Pics::new(PIC_OFFSET));

struct Pic {
    offset: u8,
    command_port: u16,
    data_port: u16,
}

impl Pic {
    fn handles_interrupt(&self, interrupt_index: u8) -> bool {
        self.offset <= interrupt_index && interrupt_index < self.offset + 8 // each PIC handles 8 interrupts
    }
}

// chained pics
pub struct Pics([Pic; 2]);

impl Pics {
    pub const fn new(offset: u8) -> Pics {
        Pics([
            Pic {
                offset: offset,
                command_port: 0x20,
                data_port: 0x21,
            },
            Pic {
                offset: offset + 8,
                command_port: 0xA0,
                data_port: 0xA1,
            },
        ])
    }

    pub fn send_eoi(&mut self, interrupt_index: u8) {
        if self.0[1].handles_interrupt(interrupt_index) {
            outb(self.0[1].command_port, 0x20); // slave
        }
        outb(self.0[0].command_port, 0x20); // master
    }
}

pub fn init() {
    let pics = &PICS.lock().0;
    let wait = || outb(0x80, 0); // delay: PIC needs time to initialize

    // original masks
    let pic1_mask = inb(pics[0].data_port);
    let pic2_mask = inb(pics[1].data_port);

    // Tell each PIC that we're going to send it a three-byte initialization sequence on its data port.
    let cmd_init = 0x11;
    outb(pics[0].command_port, cmd_init); // Command sent to begin PIC initialization.
    wait();
    outb(pics[1].command_port, cmd_init);
    wait();

    // Byte 1: Set up our base offsets.
    outb(pics[0].data_port, pics[0].offset);
    wait();
    outb(pics[1].data_port, pics[1].offset);
    wait();

    // Byte 2: Configure chaining between PIC1 and PIC2.
    outb(pics[0].data_port, 4); // Tell PIC1 that there is a PIC2 at IRQ2 (0000 0100)
    wait();
    outb(pics[1].data_port, 2); // Tell PIC2 its cascade identity (0000 0010)
    wait();

    // Byte 3: Set our mode.
    outb(pics[0].data_port, 0x01); // 8086 mode
    wait();
    outb(pics[1].data_port, 0x01);
    wait();

    // Restore our saved masks.
    outb(pics[0].data_port, pic1_mask);
    outb(pics[1].data_port, pic2_mask);
}