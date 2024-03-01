use core::fmt;
use spin::Mutex;
use volatile::Volatile;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;

lazy_static! {
    pub static ref VGA_WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::default());
}

const PROMPT_LENGTH: usize = 5;

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VgaColor {
    Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGrey,
    DarkGrey, LightBlue, LightGreen, LightCyan, LightRed, Pink, Yellow, White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct VgaColorCode(u8);

impl VgaColorCode {
    fn new(fg: VgaColor, bg: VgaColor) -> Self {
        Self((bg as u8) << 4 | fg as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct VgaChar {
    character: u8,
    color: VgaColorCode,
}

const HEIGHT: usize = 25;
const WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<VgaChar>; WIDTH]; HEIGHT],
}

pub struct VgaWriter {
    row: usize,
    column: usize,
    color: VgaColorCode,
    buffer: &'static mut Buffer,
}

impl VgaWriter {
    pub fn write_prompt(&mut self) {
        self.color = VgaColorCode::new(VgaColor::Green, VgaColor::Black);
        self.write_string("os $ ");
        self.color = VgaColorCode::new(VgaColor::White, VgaColor::Black);
    }

    pub fn set_text_color(&mut self, color: VgaColor) {
        self.color = VgaColorCode::new(color, VgaColor::Black);
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= WIDTH {
                    self.new_line();
                }
                self.write_byte_to(byte, self.row, self.column);
                self.column += 1;
            }
        }
    }

    fn write_byte_to(&mut self, byte: u8, row: usize, column: usize) {
        self.buffer.chars[row][column].write(VgaChar {
            character: byte,
            color: self.color,
        });
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => {}
            }
        }
        self.move_cursor();
    }

    fn new_line(&mut self) {
        if self.row >= HEIGHT - 1 {
            for row in 1..HEIGHT {
                for col in 0..WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(HEIGHT - 1);
        } else {
            self.row += 1;
        }
        self.column = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = VgaChar { character: b' ', color: self.color };
        for col in 0..WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn _clear_screen(&mut self) {
        for row in 0..HEIGHT {
            self.clear_row(row);
        }
    }

    fn move_cursor(&self) {
        let position = self.row * WIDTH + self.column;
        let mut cursor_control = Port::<u8>::new(0x3D4);
        let mut cursor_register = Port::<u8>::new(0x3D5);
        unsafe {
            cursor_control.write(0x0F);
            cursor_register.write((position & 0xFF) as u8);
            cursor_control.write(0x0E);
            cursor_register.write(((position >> 8) & 0xFF) as u8);
        }
    }

    pub fn delete_character(&mut self) {
        if self.column > PROMPT_LENGTH {
            self.column -= 1;
            self.write_byte_to(b' ', self.row, self.column);
            self.move_cursor();
        }
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Default for VgaWriter {
    fn default() -> Self {
        Self {
            column: 0,
            row: 0,
            color: VgaColorCode::new(VgaColor::White, VgaColor::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }
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
    VGA_WRITER.lock().write_fmt(args).expect("Printing to VGA failed");
}

pub fn clear_screen() {
    for _ in 0..HEIGHT {
        println!("");
    }
}