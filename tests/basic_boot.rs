#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic;

use blog_os::println;

bootloader_api::entry_point!(main);

fn main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output")
}
