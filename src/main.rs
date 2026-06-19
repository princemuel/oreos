#![no_std] // don't link to the standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)] // Use a custom test framework since test is in std
#![test_runner(oreos::test_runner)] // define the test runner as being a custom one.
#![reexport_test_harness_main = "test_main"] // otherwise it launches the kernel

// extern crate alloc;

// use alloc::boxed::Box;
use core::panic::PanicInfo;

use oreos::println;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("{info}",);
    loop {}
}

#[unsafe(no_mangle)] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", '!');
    // panic!("Some panic message");

    #[expect(clippy::empty_loop)]
    loop {}
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0", "wrong version");
}
