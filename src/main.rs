#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

#[unsafe(no_mangle)] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    use oreos::buffer;

    buffer::WRITER.lock().write_str("Hello again").unwrap();
    write!(buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.337).unwrap();

    #[allow(clippy::empty_loop)]
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }
