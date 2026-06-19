use core::ptr;

use spin::lazylock::LazyLock;
use spin::mutex::Mutex;

use crate::io::text;

const VGA_MMIO: usize = 0xb8000;

/// A global `Writer` instance that can be used for printing to the VGA text
/// buffer.
///
/// Used by the `print!` and `println!` macros.
pub static WRITER: LazyLock<Mutex<Writer>> = LazyLock::new(|| {
    Mutex::new(Writer {
        cursor: 0,
        code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { Buffer::at_vga_address() },
    })
});

/// The standard color palette in VGA text mode.
///
/// Colors are actually encoded such that:
/// - bytes 0-3 are for the foreground,
/// - bytes 4-7 are for the background
///
/// Each of those values come from [`Color`]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Color {
    Black = 0x00,
    Blue = 0x01,
    Green = 0x02,
    Cyan = 0x03,
    Red = 0x04,
    Magenta = 0x05,
    Brown = 0x06,
    LightGray = 0x07,
    DarkGray = 0x08,
    LightBlue = 0x09,
    LightGreen = 0x0a,
    LightCyan = 0x0b,
    LightRed = 0x0c,
    Pink = 0x0d,
    Yellow = 0x0e,
    White = 0x0f,
}

/// A combination of a foreground and a background color.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Create a new color code from background and foreground color.
    ///
    /// Each of those two colours fit into 4 bits. The background
    /// color is shifted four bits to the left then has the
    /// foreground color appended to get a valid u8 VGA color code.
    ///
    /// # Parameters
    /// * `foreground` - Foreground color,
    /// * `background` - Background color.
    #[expect(clippy::as_conversions)]
    const fn new(foreground: Color, background: Color) -> Self {
        Self(((background as u8) << 4) | (foreground as u8))
    }
}

/// A character on the screen is an ascii value and a color.
///
/// Note however that it's not a *true* ascii value, but
/// a [code page 437](https://en.wikipedia.org/wiki/Code_page_437)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ScreenChar {
    /// The printable ASCII byte (or `0xfe` as a placeholder glyph).
    ascii: u8,
    code: ColorCode,
}

impl ScreenChar {
    const fn new(ascii: u8, code: ColorCode) -> Self { Self { ascii, code } }

    #[must_use]
    const fn blank(color: ColorCode) -> Self { Self { ascii: b' ', code: color } }
}

/// The height of the text buffer (normally 25 lines).
const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer (normally 80 columns).
const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
struct Buffer {
    /// Two dimensional array of [`BUFFER_WIDTH`] by [`BUFFER_HEIGHT`]
    /// chars.
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    /// Reads a character from the VGA buffer at position `row`,`col`.
    ///
    /// Returns `None` if `row`/`col` are out of bounds.
    fn read(&self, row: usize, col: usize) -> Option<ScreenChar> {
        let cell = self.chars.get(row)?.get(col)?;

        unsafe {
            // Safety: `cell` is a memory-mapped VGA cell; the read must go
            // through hardware rather than be elided/reordered by the
            // optimizer.
            Some(ptr::read_volatile(cell))
        }
    }

    /// Writes character `ch` to `row`,`col` in the VGA buffer.
    ///
    /// Returns `false` if `row`/`col` are out of bounds; the write is
    /// skipped rather than panicking.
    fn write(&mut self, row: usize, col: usize, ch: ScreenChar) -> bool {
        let Some(row) = self.chars.get_mut(row) else { return false };
        let Some(cell) = row.get_mut(col) else { return false };

        unsafe {
            // Safety: `cell` is a memory-mapped VGA cell; the write must go
            // through hardware rather than be elided/reordered by the
            // optimizer.
            ptr::write_volatile(cell, ch);
        }
        true
    }

    /// 0xb8000 is the standard physical address of the VGA
    /// text-mode buffer on x86, identity-mapped by the bootloader before
    /// this runs. `Buffer` has alignment 1, so no alignment requirement
    /// applies. This is the *only* place a reference to this address is
    /// ever created; all access goes through the single [`WRITER`]
    /// static guarded by `Mutex`, so the `&'static mut` here is never
    /// aliased.
    /// # Safety
    /// Caller must guarantee no other live reference to the VGA buffer
    /// exists for the duration of the returned reference's lifetime.
    unsafe fn at_vga_address() -> &'static mut Self {
        #[expect(clippy::as_conversions)]
        let ptr = VGA_MMIO as *mut Self;
        unsafe { &mut *ptr }
    }
}

/// A writer type that allows writing ASCII bytes and strings to an underlying
/// `Buffer`.
///
/// Wraps lines at [`BUFFER_WIDTH`]. Supports newline characters and implements
/// the `core::fmt::Write` trait.
pub struct Writer {
    /// Current column position on the last row.
    cursor: usize,
    /// The current color code.
    code: ColorCode,
    /// The VGA buffer where text is displayed.
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at [`BUFFER_WIDTH`]. Supports the `\n` newline character.
    /// Does **not** support strings with non-ASCII characters, since they
    /// can't be printed in the VGA text mode.
    pub fn write_string(&mut self, value: impl AsRef<str>) {
        for c in value.as_ref().chars() {
            self.write_char(c);
        }
    }

    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at [`BUFFER_WIDTH`]. Supports the `\n` newline and `\t` tab
    /// characters.
    fn write_char(&mut self, value: char) {
        match value {
            '\n' => self.new_line(),
            '\t' => {
                while !self.cursor.is_multiple_of(8) {
                    self.write_byte(b' ');
                }
            }
            value => {
                // spades symbol for unknown character
                let byte = text::encode(value).unwrap_or(6);
                self.write_byte(byte);
            }
        }
    }

    fn write_byte(&mut self, value: u8) {
        if self.cursor >= BUFFER_WIDTH {
            self.new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.cursor;
        let code = self.code;

        debug_assert!(
            self.buffer.write(row, col, ScreenChar::new(value, code)),
            "write_byte: cursor {col} should always be < BUFFER_WIDTH here"
        );

        self.cursor += 1;
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let Some(ch) = self.buffer.read(row, col) else {
                    debug_assert!(false, "new_line: ({row}, {col}) should always be in bounds");
                    continue;
                };

                let in_bounds = self.buffer.write(row - 1, col, ch);
                debug_assert!(
                    in_bounds,
                    "new_line: ({row}, {col}) should always be in bounds",
                    row = row - 1
                );
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.cursor = 0;
    }

    /// Clears a row by overwriting it with blank (space) characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar::blank(self.code);
        for col in 0..BUFFER_WIDTH {
            let in_bounds = self.buffer.write(row, col, blank);
            debug_assert!(in_bounds, "clear_row: ({row}, {col}) should always be in bounds");
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the VGA text
/// buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::output::vga::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA
/// text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer through the global
/// `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments<'_>) {
    use core::fmt::Write as _;

    {
        let mut w = WRITER.lock();
        let _ = w.write_fmt(args);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// # Panics
    /// If the test fails…
    #[test_case]
    fn direct_output() {
        use core::fmt::Write;

        let string = "Some test string that fits on a single line";
        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writeln!(writer, "\n{string}").expect("writeln failed");
            for (i, character) in string.chars().enumerate() {
                let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
                assert_eq!(
                    char::from(screen_char.ascii_character),
                    character,
                    "mismatch between characters"
                );
            }
        });
    }

    #[test_case]
    fn println() {
        println!("test_println! output");
    }

    #[test_case]
    fn print_many_lines() { (0..200).for_each(|_| println!("print_many_lines output")); }

    #[test_case]
    fn print_long_line() {
        let string = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        println!("{string}{string}{string}{string}");
    }

    #[expect(clippy::missing_panics_doc)]
    #[test_case]
    fn check_output() {
        let string = "Some test string that fits on a single line";
        println!("{}", string);
        string.chars().enumerate().for_each(|(i, character)| {
            let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(
                char::from(screen_char.ascii_character),
                character,
                "printed values are different…"
            );
        });
    }
}
