use std::sync::atomic::Ordering;

use crate::lb::backend::Backend;

pub struct ConnectionGuard<'a> {
    backend: &'a Backend,
}

impl<'a> ConnectionGuard<'a> {
    pub fn new(backend: &'a Backend) -> Self {
        backend.active_connections.fetch_add(1, Ordering::Relaxed);

        Self { backend }
    }
}

impl<'a> Drop for ConnectionGuard<'a> {
    fn drop(&mut self) {
        self.backend
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
    }
}
