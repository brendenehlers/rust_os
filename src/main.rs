#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{panic, sync::atomic::Ordering};

extern crate alloc;

use blog_os::{
    SHUTDOWN, exit_qemu, task::{Task, executor::Executor, keyboard}, thread::{self, yield_now},
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
        executor.spawn(Task::new(keyboard::handle_scancodes()));
        executor.run();
    });

    thread::spawn(|| { 
        call_me(0);
    });

    loop { 
        if SHUTDOWN.load(Ordering::Relaxed) {
            serial_println!("goodbye");
            exit_qemu(blog_os::QemuExitCode::Success);
        }
        yield_now(); 
    }
}

fn call_me(i: usize) {
    println!("{}", i);
    call_me(i + 1);
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
