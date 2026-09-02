use alloc::{boxed::Box, collections::VecDeque};
use conquer_once::spin::OnceCell;
use spin::Mutex;

use crate::println;

use super::Thread;

pub(super) static SCHEDULER: OnceCell<Mutex<Scheduler>> = OnceCell::uninit();

pub(super) struct Scheduler {
    pub current: Box<Thread>,
    pub queue: VecDeque<Box<Thread>>,
    _private: (), // prevents creating this struct outside this module
}

impl Scheduler {
    pub(super) fn init() {
        SCHEDULER
            .try_init_once(|| {
                Mutex::new(Scheduler {
                    current: Box::new(Thread::bootstrap()),
                    queue: VecDeque::new(),
                    _private: (),
                })
            })
            .expect("failed to init thread scheduler");
        println!("thread scheduler initialized");
    }
}
