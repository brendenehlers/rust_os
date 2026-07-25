#![no_std]
#![no_main]

use core::panic;

mod vga_buffer;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("hello, world{}", "!");

    loop {}
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
