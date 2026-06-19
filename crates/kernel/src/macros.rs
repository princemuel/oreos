/// Like the `print!` macro in the standard library, but prints to the VGA text
/// buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::macros::_print(format_args!($($arg)*)));
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

    use crate::buffer::WRITER;

    {
        let mut w = WRITER.lock();
        let _ = w.write_fmt(args);
    }
}
