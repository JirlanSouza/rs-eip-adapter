use std::sync::atomic::{AtomicU32, Ordering};

pub struct SessionManager {
    next: AtomicU32,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            next: AtomicU32::new(1),
        }
    }

    pub fn new_session(&self) -> u32 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }
}
