use crate::lb::{backend::Backend, strategy::SelectionStrategy};
use anyhow::{Result, anyhow};
use std::sync::atomic::Ordering;

pub struct LeastConnections;

impl LeastConnections {
    pub fn new() -> Self {
        Self
    }
}

impl SelectionStrategy for LeastConnections {
    fn select<'a>(&self, backends: &'a [Backend]) -> Result<&'a Backend> {
        backends
            .iter()
            .filter(|backend| backend.is_alive.load(Ordering::Relaxed))
            .min_by_key(|backend| backend.active_connections.load(Ordering::Relaxed))
            .ok_or_else(|| anyhow!("No backends to route request to"))
    }
}
