#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic;

mod vga_buffer;
mod serial;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("hello, world{}", "!");

    blog_os::init();

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("it didn't crash!");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
