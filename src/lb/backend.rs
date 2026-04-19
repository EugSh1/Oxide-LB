use std::{
    net::SocketAddr,
    sync::atomic::{AtomicBool, AtomicU64},
};

#[derive(Debug)]
pub struct Backend {
    pub addr: SocketAddr,
    pub is_alive: AtomicBool,
    pub active_connections: AtomicU64,
}
