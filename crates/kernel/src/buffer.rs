use core::ptr;

use spin::{LazyLock, Mutex};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorCode(u8);

impl ColorCode {
    #[expect(clippy::as_conversions)]
    const fn new(foreground: Color, background: Color) -> Self {
        Self(((background as u8) << 4) | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenChar {
    /// The printable ASCII byte (or `0xfe` as a placeholder glyph).
    ascii: u8,
    code: ColorCode,
}

impl ScreenChar {
    const fn new(ascii: u8, code: ColorCode) -> Self { Self { ascii, code } }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    /// Writes character `ch` to `row`,`col` in the VGA buffer.
    ///
    /// Returns `false` if `row`/`col` are out of bounds; the write is
    /// skipped rather than panicking.
    fn write(&mut self, row: usize, col: usize, ch: ScreenChar) -> bool {
        let Some(row) = self.chars.get_mut(row) else { return false };
        let Some(cell) = row.get_mut(col) else { return false };

        unsafe {
            // UNSAFE: `cell` is a memory-mapped VGA cell; the write must go
            // through hardware rather than be elided/reordered by the
            // optimizer.
            ptr::write_volatile(cell, ch);
        }
        true
    }

    /// Reads a character from the VGA buffer at position `row`,`col`.
    ///
    /// Returns `None` if `row`/`col` are out of bounds.
    fn read(&self, row: usize, col: usize) -> Option<ScreenChar> {
        let cell = self.chars.get(row)?.get(col)?;

        unsafe {
            // UNSAFE: `cell` is a memory-mapped VGA cell; the read must go
            // through hardware rather than be elided/reordered by the
            // optimizer.
            Some(ptr::read_volatile(cell))
        }
    }

    /// # Safety
    /// Caller must guarantee no other live reference to the VGA buffer
    /// exists for the duration of the returned reference's lifetime.
    unsafe fn at_vga_address() -> &'static mut Buffer {
        // SAFETY: 0xb8000 is the standard physical address of the VGA
        // text-mode buffer on x86, identity-mapped by the bootloader before
        // this runs. `Buffer` has alignment 1, so no alignment requirement
        // applies. This is the *only* place a reference to this address is
        // ever created; all access goes through this single `WRITER`
        // static guarded by `Mutex`, so the `&'static mut` here is never
        // aliased.
        #[expect(clippy::as_conversions)]
        let ptr = 0xb8000 as *mut Buffer;

        unsafe { &mut *ptr }
    }
}

pub static WRITER: LazyLock<Mutex<Writer>> = LazyLock::new(|| {
    Mutex::new(Writer {
        pos: 0,
        code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { Buffer::at_vga_address() },
    })
});

pub struct Writer {
    pos: usize,
    code: ColorCode,
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
                let code = self.code;

                debug_assert!(
                    self.buffer.write(row, col, ScreenChar::new(byte, code)),
                    "write_byte: pos {col} should always be < BUFFER_WIDTH here"
                );

                self.pos += 1;
            }
        }
    }

    /// Scrolls every row up by one, dropping row `0` and leaving a blank
    /// row at the bottom.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let Some(ch) = self.buffer.read(row, col) else {
                    debug_assert!(false, "new_line: ({row}, {col}) should always be in bounds");
                    continue;
                };
                debug_assert!(
                    self.buffer.write(row - 1, col, ch),
                    "new_line: ({}, {col}) should always be in bounds",
                    row - 1
                );
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.pos = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar::new(b' ', self.code);
        for col in 0..BUFFER_WIDTH {
            debug_assert!(
                self.buffer.write(row, col, blank),
                "clear_row: ({row}, {col}) should always be in bounds"
            );
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
