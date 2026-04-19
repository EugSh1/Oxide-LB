use crate::lb::{backend::Backend, strategy::SelectionStrategy};
use anyhow::{Result, bail};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct RoundRobin {
    current_backend_index: AtomicUsize,
}

impl RoundRobin {
    pub fn new() -> Self {
        Self {
            current_backend_index: AtomicUsize::new(0),
        }
    }
}

impl SelectionStrategy for RoundRobin {
    fn select<'a>(&self, backends: &'a [Backend]) -> Result<&'a Backend> {
        let backends_count = backends.len();

        if backends_count == 0 {
            bail!("No backends provided");
        }

        for _ in 0..backends_count {
            let index = self.current_backend_index.fetch_add(1, Ordering::Relaxed);

            let backend = &backends[index % backends_count];

            if backend.is_alive.load(Ordering::Relaxed) {
                return Ok(backend);
            }
        }

        bail!("No backends to route request to");
    }
}
