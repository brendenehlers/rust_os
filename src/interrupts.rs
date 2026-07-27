use x86_64::structures::idt;

use crate::println;

lazy_static::lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: idt::InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_interrupt() {
    x86_64::instructions::interrupts::int3();
}
