#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{panic};

extern crate alloc;

use alloc::{
    rc::{self, Rc},
    vec,
};
use blog_os::{
    allocator, task::{Task, executor::Executor, keyboard}, thread::{self},
};

mod serial;
mod vga_buffer;

bootloader::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
    use blog_os::memory;

    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    thread::init();

    println!("hello, world{}", "!");
    blog_os::init();

    let heap_value = alloc::boxed::Box::new(41);
    println!("heap_value at {:p}", heap_value);

    let mut vec = vec::Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let ref_counted = rc::Rc::new(vec![1, 2, 3]);
    let clone_ref = ref_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&clone_ref)
    );
    core::mem::drop(ref_counted);
    println!("ref count is {} now", Rc::strong_count(&clone_ref));

    thread::spawn(|| {
        let mut i = 0;
        loop {
            println!("thread a ran {} times", i);
            i += 1;
            for _ in 0..1000000 {}

        }
    });
    thread::spawn(|| {
        let mut i = 0;
        loop {
            println!("thread b ran {} times", i);
            i += 1;
            for _ in 0..1000000 {}
        }
    });
    println!("back in kernel_main");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

extern "C" fn thread_a() -> ! {
    let mut i = 0;
    loop {
        println!("thread a ran {} times", i);
        i += 1;
        for _ in 0..1000000 {}

    }
}

extern "C" fn thread_b() -> ! {
    let mut i = 0;
    loop {
        println!("thread b ran {} times", i);
        i += 1;
        for _ in 0..1000000 {}
    }
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
