use core::fmt;
use spin::Mutex;
use volatile::Volatile;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref VGA_WRITER: Mutex<VgaWriter> = Mutex::new(VgaWriter::default());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VgaColor {
    Black, _Blue, _Green, _Cyan, _Red, _Magenta, _Brown, _LightGrey,
    _DarkGrey, _LightBlue, _LightGreen, _LightCyan, _LightRed, _Pink, _Yellow, White,
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
    column_position: usize,
    color: VgaColorCode,
    buffer: &'static mut Buffer,
}

impl VgaWriter {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            _ => {
                if self.column_position >= WIDTH {
                    self.new_line();
                }
                let row = HEIGHT - 1;
                self.buffer.chars[row][self.column_position].write(VgaChar { character: byte, color: self.color });
                self.column_position += 1;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
    }

    fn new_line(&mut self) {
        for row in 1..HEIGHT {
            for col in 0..WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = VgaChar { character: b' ', color: self.color };
        for col in 0..WIDTH {
            self.buffer.chars[row][col].write(blank);
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
            column_position: 0,
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