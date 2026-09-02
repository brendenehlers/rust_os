use core::{alloc::Layout, arch::naked_asm, mem};

use alloc::{alloc::{alloc_zeroed, handle_alloc_error}, boxed::Box};
use x86_64::instructions::interrupts;

mod scheduler;

use scheduler::{Scheduler, SCHEDULER};

#[derive(Debug)]
pub struct Thread {
    status: ThreadStatus,

    // pointer within `_stack` to the current frame of execution
    stack_pointer: *mut u8,

    // a `Thread` maintains the reference to the stack on the heap
    // so the stack is dealloced when the thread goes out of scope
    _stack: Option<Box<Stack>>, // None -> bootloader stack
}

#[derive(Debug, PartialEq)]
pub enum ThreadStatus {
    Ready,
    Running,
    Finished,
}

extern "C" fn thread_shim<F: FnOnce() + Send + 'static>(arg: *mut u8) -> ! {
    let f = unsafe { Box::from_raw(arg as *mut F) };
    f();
    thread_exit();
}

fn thread_exit() -> ! {
    interrupts::without_interrupts(|| {
        SCHEDULER.try_get().expect("thread scheduler not init")
            .lock()
            .current
            .status = ThreadStatus::Finished;
    });
    loop { yield_now(); }
}

impl Thread {
    // creates a new thread that needs to be written to before running.
    pub fn bootstrap() -> Self {
        Self { 
            stack_pointer: core::ptr::null_mut(), 
            status: ThreadStatus::Running, // bootloader thread is already running
            _stack: None 
        }
    }

    /// private helper function to create a new thread
    fn new_c(entry: extern "C" fn() -> !) -> Self {
        let mut stack = Stack::forge(entry as usize as u64, 0);
        let stack_pointer = &mut stack.0[SP_INDEX] as *mut u64 as *mut u8;
        Self { 
            stack_pointer, 
            status: ThreadStatus::Ready,
            _stack: Some(stack) 
        }
    }

    /// creates a new thread struct with the entry function wrapped in an
    /// `extern "C"` wrapper shim so the entry is picked up when the thread
    /// is run
    fn new<F: FnOnce() + Send + 'static>(f: F) -> Self {
        let arg = Box::into_raw(Box::new(f)) as *mut u8;
        let entry = thread_shim::<F> as *const () as u64;
        let mut stack = Stack::forge(entry, arg as u64);
        let stack_pointer = &mut stack.0[SP_INDEX] as *mut u64 as *mut u8;
        Thread { 
            stack_pointer, 
            status: ThreadStatus::Ready,
            _stack: Some(stack) 
        }
    }
}

// SAFETY: `Thread` owns the stack allocation exclusively. 
unsafe impl Send for Thread {}

// externally-available function to create a new thread on the scheduler
pub fn spawn_c(entry: extern "C" fn() -> !) {
    let t = Box::new(Thread::new_c(entry));
    interrupts::without_interrupts(|| {
        SCHEDULER.try_get().expect("thread scheduler not init")
            .lock()
            .queue
            .push_back(t);
    });
}

/// Creates a new thread on the scheduler running the provided function
pub fn spawn<F: FnOnce() + Send + 'static>(f: F) {
    let t = Box::new(Thread::new(f));
    interrupts::without_interrupts(|| {
        SCHEDULER.try_get().expect("thread scheduler not init")
            .lock()
            .queue
            .push_back(t);
    });
}

const STACK_SLOTS: usize = 2_048; // 16kib stack
const SP_INDEX: usize = STACK_SLOTS - 8;
const R13_INDEX: usize = SP_INDEX + 2;
const R12_INDEX: usize = SP_INDEX + 3;
const ENTRY_INDEX: usize = SP_INDEX + 6;

#[repr(align(16))]
#[derive(Debug)]
struct Stack([u64; STACK_SLOTS]);

impl Stack {
    /// pop r15
    /// pop r14
    /// pop r13
    /// pop r12
    /// pop rbx
    /// pop rbp
    /// ret
    fn forge(entry_ptr: u64, arg: u64) -> Box<Self> {
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

        // set the r12 register used by thread_entry_trampoline
        stack.0[R12_INDEX] = entry_ptr;
        // set arbitrary data into r13 for the trampoline
        stack.0[R13_INDEX] = arg;

        // set the entrypoint that's used by `ret`
        let trampoline_ptr = thread_entry_trampoline as *const() as usize as u64;
        stack.0[ENTRY_INDEX] = trampoline_ptr;

        stack
    }
}

/// Yields the currently executing thread so another thread can work.
pub fn yield_now() {
    interrupts::without_interrupts(|| {
        let (old_sp, new_sp) = {
            let mut scheduler = SCHEDULER
                .try_get()
                .expect("failed to acquire the thread scheduler")
                .lock();

            // find next ready thread and drop any finished threads
            // TODO: move this into own function
            let mut next: Option<Box<Thread>> = None;
            while let Some(n) = scheduler.queue.pop_front() {
                match n.status {
                    ThreadStatus::Ready => {
                        next = Some(n);
                        break;
                    }
                    ThreadStatus::Finished => {
                        // drop the finished thread, thus freeing the stack
                        drop(n);
                    }
                    ThreadStatus::Running => {
                        panic!("more than one thread in running state");
                    }
                }
            }

            let mut old = match next {
                Some(mut next) => {
                    next.status = ThreadStatus::Running;
                    mem::replace(&mut scheduler.current, next)
                },
                None => return // nothing to switch to
            };

            if old.status != ThreadStatus::Finished {
                old.status = ThreadStatus::Ready;
            }
            scheduler.queue.push_back(old);
            // unwrap safe here bc we just pushed the element to the back
            let old_sp = &mut scheduler
                .queue
                .back_mut()
                .unwrap()
                .stack_pointer as *mut *mut u8;
            let new_sp = scheduler.current.stack_pointer;
            (old_sp, new_sp)
        };

        // we MUST drop the lock on SCHEDULER before calling switch_context
        unsafe { switch_context(old_sp, new_sp); }
    });
}

/// Switches from one thread to another using the stack pointers provided as args
#[unsafe(naked)]
unsafe extern "C" fn switch_context(old: *mut *mut u8, new: *mut u8) {
    naked_asm!(
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov [rdi], rsp", // rdi automatically stores the first function arg
        "mov rsp, rsi", // rsi automatically stores the second function arg
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "ret",
    );
}

#[unsafe(naked)]
unsafe extern "C" fn thread_entry_trampoline() -> ! {
    naked_asm!(
        "sti",
        "mov rdi, r13",
        "jmp r12",
    )
}


pub fn init() {
    Scheduler::init();
}
