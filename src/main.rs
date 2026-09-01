#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{panic};

extern crate alloc;

use blog_os::{
    task::{Task, executor::Executor, keyboard}, thread::{self, yield_now},
};

mod serial;
mod vga_buffer;

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {

    println!("hello, world{}", "!");
    blog_os::init(boot_info);

    #[cfg(test)]
    test_main();

    thread::spawn(|| {
        let mut executor = Executor::new();
        executor.spawn(Task::new(keyboard::print_keypresses()));
        executor.run();
    });

    loop { yield_now(); }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
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
