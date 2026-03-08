use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
pub struct SessionManager {
    next: AtomicU32,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_session(&self) -> u32 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }
}
