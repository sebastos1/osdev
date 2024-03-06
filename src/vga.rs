use core::fmt;
use spin::Mutex;
use volatile::Volatile;
use lazy_static::lazy_static;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        color: VgaColor::new(Color::White, Color::Black),
        column: 0,
        row: 0,
    });
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<Char>; VGA_WIDTH]; VGA_HEIGHT],
}

pub struct Writer {
    buffer: &'static mut Buffer,
    color: VgaColor,
    column: usize,
    row: usize,
}

impl Writer {
    // https://wiki.osdev.org/Text_Mode_Cursor#Moving_the_Cursor_2
    fn move_cursor(&self) {
        let pos = self.row * VGA_WIDTH + self.column;
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);
            outb(0x3D4, 0x0E);
            outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= VGA_WIDTH {
                    self.new_line();
                }
                self.buffer.chars[self.row][self.column].write(Char {
                    character: byte,
                    color: self.color
                });
                self.column += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => {} // not printable
            }
        }
        self.move_cursor();
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..VGA_WIDTH {
            self.buffer.chars[row][col].write(Char {
                character: b' ',
                color: self.color,
            });
        }
    }

    fn new_line(&mut self) {
        if self.row >= VGA_HEIGHT - 1 {
            for row in 1..VGA_HEIGHT {
                for col in 0..VGA_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(VGA_HEIGHT - 1);
        } else {
            self.row += 1;
        }
        self.column = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[allow(unused)]
#[repr(u8)]
pub enum Color {
    Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGrey, DarkGrey, 
    LightBlue, LightGreen, LightCyan, LightRed, Pink, Yellow, White,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct VgaColor(u8);

impl VgaColor {
    fn new(foreground: Color, background: Color) -> VgaColor {
        VgaColor((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Char {
    character: u8,
    color: VgaColor,
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).expect("Printing to VGA failed");
}

// move this later on
unsafe fn outb(port: u16, value: u8) {
    use core::arch::asm;
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
    );
}

pub fn clear_screen() {
    for _ in 0..VGA_HEIGHT {
        println!("");
    }
}