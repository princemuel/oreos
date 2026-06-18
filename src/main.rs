#![no_std]
#![no_main]

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
    println!("Hello World{}", "!");
    // panic!("Some panic message");

    #[expect(clippy::empty_loop)]
    loop {}
}
