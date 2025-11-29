use core::ptr;

use lazy_static::lazy_static;
use spin::Mutex;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorCode(u8);
impl ColorCode {
    const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenChar {
    /// The ASCII character
    char: u8,
    code: ColorCode,
}

impl ScreenChar {
    const fn new(ch: u8, code: ColorCode) -> Self { Self { char: ch, code } }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}
impl Buffer {
    /// Writes character `c` to `row`,`col` in the VGA buffer.
    fn write(&mut self, row: usize, col: usize, ch: ScreenChar) {
        unsafe {
            // UNSAFE: all pointers in `chars` point to a valid ScreenChar in the VGA
            // buffer.
            ptr::write_volatile(&mut self.chars[row][col], ch);
        }
    }

    /// Reads a character from the VGA buffer at position `row`,`col`.
    fn read(&self, row: usize, col: usize) -> ScreenChar {
        unsafe {
            // UNSAFE: all pointers in `chars` point to a valid ScreenChar in the VGA
            // buffer.
            ptr::read_volatile(&self.chars[row][col])
        }
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        pos:    0,
        code:   ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}
pub struct Writer {
    pos:    usize,
    code:   ColorCode,
    buffer: &'static mut Buffer,
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
                if self.pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.pos;

                let color_code = self.code;

                self.buffer
                    .write(row, col, ScreenChar::new(byte, color_code));

                self.pos += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let ch = self.buffer.read(row, col);
                self.buffer.write(row, col, ch);
            }
        }
        self.clear_row(BUFFER_WIDTH - 1);
        self.pos = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar::new(b' ', self.code);
        for col in 0..BUFFER_WIDTH {
            self.buffer.write(row, col, blank);
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
