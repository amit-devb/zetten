use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Progress {
    total: usize,
    running: AtomicUsize,
    done: AtomicUsize,
}

impl Progress {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            running: AtomicUsize::new(0),
            done: AtomicUsize::new(0),
        }
    }

    pub fn start_task(&self) {
        self.running.fetch_add(1, Ordering::SeqCst);
    }

    pub fn finish_task(&self) {
        self.running.fetch_sub(1, Ordering::SeqCst);
        self.done.fetch_add(1, Ordering::SeqCst);
    }

    pub fn snapshot(&self) -> (usize, usize, usize) {
        (
            self.running.load(Ordering::SeqCst),
            self.done.load(Ordering::SeqCst),
            self.total,
        )
    }
}
