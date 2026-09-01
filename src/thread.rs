use core::alloc::Layout;

use alloc::{alloc::{alloc_zeroed, handle_alloc_error}, boxed::Box};

#[derive(Debug)]
pub struct Thread {
    pub stack_pointer: *mut u8,
    pub stack: Box<Stack>, // keeps the memory reference to the stack alive
}

impl Thread {
    pub fn spawn_c(entry: extern "C" fn() -> !) -> Self {
        let mut stack = Stack::forge(entry as usize as u64);
        let stack_pointer = &mut stack.0[SP_INDEX] as *mut u64 as *mut u8;
        Self { stack_pointer, stack: stack }
    }
}

const STACK_SLOTS: usize = 2_048; // 16kib stack
const SP_INDEX: usize = STACK_SLOTS - 8;
const ENTRY_INDEX: usize = STACK_SLOTS - 2;

#[repr(align(16))]
#[derive(Debug)]
pub struct Stack(pub [u64; STACK_SLOTS]);

impl Stack {
    /// pop r15
    /// pop r14
    /// pop r13
    /// pop r12
    /// pop rbx
    /// pop rbp
    /// ret
    fn forge(entry_ptr: u64) -> Box<Self> {
        let layout = Layout::new::<Stack>();
        // set stack to zeros so we don't have messed up data initially
        let stack_ptr = unsafe { alloc_zeroed(layout) } as *mut Stack;
        if stack_ptr.is_null() {
            handle_alloc_error(layout);
        }

        let mut stack = unsafe {
            Box::from_raw(stack_ptr)
        };

        let sp = &stack.0[SP_INDEX] as *const u64 as usize;
        assert_eq!(sp % 16, 0);

        // set the entrypoint that's used by `ret`
        stack.0[ENTRY_INDEX] = entry_ptr;

        stack
    }
}
