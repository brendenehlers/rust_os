use alloc::boxed;
use core::{
    future, pin,
    sync::atomic::{AtomicU64, Ordering},
    task,
};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

pub struct Task {
    id: TaskId,
    future: pin::Pin<boxed::Box<dyn future::Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl future::Future<Output = ()> + 'static) -> Self {
        Task {
            id: TaskId::new(),
            future: boxed::Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut task::Context) -> task::Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
