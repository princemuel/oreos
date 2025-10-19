pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        code:            ColorCode::new(Color::Yellow, Color::Black),
        buffer:          unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
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
pub struct ScreenCharacter {
    /// The ASCII character
    char: u8,
    code: ColorCode,
}

impl ScreenCharacter {
    fn new(character: u8, code: ColorCode) -> Self { Self { char: character, code } }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    chars: [[ScreenCharacter; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    code:            ColorCode,
    buffer:          &'static mut Buffer,
}

impl Writer {
    pub fn new(column_position: usize, code: ColorCode, buffer: &'static mut Buffer) -> Self {
        Self { column_position, code, buffer }
    }

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
                self.buffer.chars[row][col] = ScreenCharacter::new(byte, color_code);
                self.column_position += 1;
            },
        }
    }

    fn new_line(&mut self) { todo!() }
}
