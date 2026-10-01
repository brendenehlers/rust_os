use core::mem;

use alloc::{boxed::Box, collections::VecDeque};
use conquer_once::spin::OnceCell;
use spin::Mutex;
use x86_64::instructions::interrupts;

use crate::println;

use super::{Thread, ThreadStatus, switch_context};

static SCHEDULER: OnceCell<Mutex<Scheduler>> = OnceCell::uninit();

struct Scheduler {
    current: Box<Thread>,
    queue: VecDeque<Box<Thread>>,
}

pub(super) fn init() {
    SCHEDULER
        .try_init_once(|| {
            Mutex::new(Scheduler {
                current: Box::new(Thread::bootstrap()),
                queue: VecDeque::new(),
            })
        })
        .expect("failed to init thread scheduler");
    println!("thread scheduler initialized");
}

/// Yields the currently executing thread so another thread can work.
pub(super) fn yield_now() {
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
                }
                None => return, // nothing to switch to
            };

            if old.status != ThreadStatus::Finished {
                old.status = ThreadStatus::Ready;
            }
            scheduler.queue.push_back(old);
            // unwrap safe here bc we just pushed the element to the back
            let old_sp = &mut scheduler.queue.back_mut().unwrap().stack_pointer as *mut *mut u8;
            let new_sp = scheduler.current.stack_pointer;
            (old_sp, new_sp)
        };

        // we MUST drop the lock on SCHEDULER before calling switch_context
        unsafe {
            switch_context(old_sp, new_sp);
        }
    });
}

pub(super) fn enqueue(t: Box<Thread>) {
    interrupts::without_interrupts(|| {
        SCHEDULER
            .try_get()
            .expect("thread scheduler not init")
            .lock()
            .queue
            .push_back(t);
    });
}

pub(super) fn finish_current() {
    interrupts::without_interrupts(|| {
        SCHEDULER
            .try_get()
            .expect("thread scheduler not init")
            .lock()
            .current
            .status = ThreadStatus::Finished;
    });
}
