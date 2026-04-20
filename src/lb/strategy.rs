use crate::lb::{
    backend::Backend,
    strategies::{least_connections::LeastConnections, round_robin::RoundRobin},
};
use anyhow::Result;

pub enum SelectionStrategy {
    RoundRobin(RoundRobin),
    LeastConnections(LeastConnections),
}

impl SelectionStrategy {
    pub fn select<'a>(&self, backends: &'a [Backend]) -> Result<&'a Backend> {
        match self {
            Self::RoundRobin(strategy) => strategy.select(backends),
            Self::LeastConnections(strategy) => strategy.select(backends),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::RoundRobin(_) => "Round Robin",
            Self::LeastConnections(_) => "Least Connections",
        }
    }
}
