use lazy_static::lazy_static;
use spin::Mutex;
use volatile::{VolatilePtr, VolatileRef};

pub fn print_something() {
    use core::fmt::Write;

    let mut w = Writer {
        column_position: 0,
        code:            ColorCode::new(Color::Yellow, Color::Black),
        buffer:          unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    w.write_byte(b'H');
    w.write_string("ello! ");
    write!(w, "The numbers are {} and {}", 42, 1.0 / 3.0).unwrap();
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorCode(u8);
impl ColorCode {
    fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenChar {
    /// The ASCII character
    char: u8,
    code: ColorCode,
}

impl ScreenChar {
    fn new(character: u8, code: ColorCode) -> Self { Self { char: character, code } }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    chars: [[VolatilePtr<'static, ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        code:            ColorCode::new(Color::Yellow, Color::Black),
        buffer:          unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}
pub struct Writer {
    column_position: usize,
    code:            ColorCode,
    buffer:          &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, value: impl AsRef<str>) {
        for byte in value.as_ref().bytes() {
            match byte {
                // it is a printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // it is not part of the printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, value: u8) {
        match value {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.code;

                {
                    let item = self.buffer.chars[row][col];
                    item.write(ScreenChar::new(byte, color_code));
                }

                // self.buffer.chars[row][col] = ScreenChar::new(byte, color_code);

                self.column_position += 1;
            },
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar::new(b' ', self.code);
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
