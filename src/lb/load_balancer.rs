use crate::lb::{backend::Backend, config::Config, connection_guard::ConnectionGuard};
use anyhow::{Context, Result, anyhow};
use std::{
    net::SocketAddr,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    signal::unix::SignalKind,
};

struct InnerState {
    config: Config,
}

#[derive(Clone)]
pub struct LoadBalancer {
    inner: Arc<InnerState>,
}

impl LoadBalancer {
    pub fn new(config: Config) -> Self {
        Self {
            inner: Arc::new(InnerState { config }),
        }
    }

    pub async fn run(self) -> Result<()> {
        tracing::info!(
            strategy = self.inner.config.strategy.name(),
            bind_addr = %self.inner.config.bind_addr,
            backends_count = self.inner.config.backend_addresses.len(),
            "Oxide-LB started"
        );

        #[cfg(unix)]
        let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())
            .context("Failed to install SIGTERM handler")?;

        let listener = TcpListener::bind(&self.inner.config.bind_addr).await?;

        let lb = self.clone();

        tokio::spawn(async move {
            lb.run_health_check_worker().await;
        });

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received Ctrl+C, gracefully shutting down");
                    break;
                }
                _ = async {
                    #[cfg(unix)]
                    { sigterm.recv().await }
                    #[cfg(not(unix))]
                    { std::future::pending::<Option<()>>().await }
                } => {
                    tracing::info!("Received SIGTERM, gracefully shutting down");
                    break;
                }
                res = listener.accept() => {
                    let (socket, _) = match res {
                        Ok(conn) => conn,
                        Err(error) => {
                            tracing::warn!(error = %error, "Failed to accept connection, continuing...");
                            continue;
                        }
                    };


                    let lb = self.clone();

                    tokio::spawn(async move {
                        if let Err(error) = lb.handle_connection(socket).await {
                            tracing::error!(
                                error = %error,
                                details = ?error,
                                "Failed to handle connection"
                            );
                        }
                    });
                }
            }
        }

        match tokio::time::timeout(Duration::from_secs(30), self.shutdown_gracefully()).await {
            Ok(_) => tracing::info!("Graceful shutdown complete. All connections closed."),
            Err(_) => tracing::warn!("Graceful shutdown timeout reached. Force stopping."),
        }

        Ok(())
    }

    async fn handle_connection(&self, mut socket: TcpStream) -> Result<()> {
        let target_backend = self.next_backend()?;

        // connection guard will automatically decrement active connections counter on drop
        let _connection_guard = ConnectionGuard::new(target_backend);

        tracing::debug!(
            "New connection - Routing to backend {}",
            target_backend.addr
        );

        let mut backend_socket = tokio::time::timeout(
            Duration::from_secs(5),
            TcpStream::connect(target_backend.addr),
        )
        .await
        .map_err(|_| anyhow!("Backend connection timed out"))??;

        match tokio::io::copy_bidirectional(&mut socket, &mut backend_socket).await {
            Ok((client_to_backend_bytes, backend_to_client_bytes)) => {
                tracing::debug!(
                    "Connection closed. Client -> Backend: {} bytes, Backend -> Client: {} bytes",
                    client_to_backend_bytes,
                    backend_to_client_bytes
                );
            }
            Err(error) => {
                tracing::debug!(%error, "Connection forcefully closed during transfer");
            }
        }

        Ok(())
    }

    fn next_backend(&self) -> Result<&Backend> {
        self.inner
            .config
            .strategy
            .select(&self.inner.config.backend_addresses)
    }

    async fn run_health_check_worker(&self) {
        tracing::info!("Health check worker started");

        loop {
            self.check_all_backends();
            tokio::time::sleep(self.inner.config.health_check_interval).await;
        }
    }

    fn check_all_backends(&self) {
        for i in 0..self.inner.config.backend_addresses.len() {
            let lb = self.clone();

            tokio::spawn(async move {
                let backend = &lb.inner.config.backend_addresses[i];
                let is_alive =
                    tokio::time::timeout(Duration::from_secs(5), lb.is_backend_alive(backend.addr))
                        .await
                        .unwrap_or(false);

                let was_alive = backend.is_alive.load(Ordering::Relaxed);

                if is_alive && !was_alive {
                    tracing::info!("Backend {} is back online", backend.addr)
                } else if !is_alive && was_alive {
                    tracing::warn!("Backend {} went offline", backend.addr)
                }

                backend.is_alive.store(is_alive, Ordering::Relaxed);
            });
        }
    }

    async fn is_backend_alive(&self, backend_addr: SocketAddr) -> bool {
        TcpStream::connect(backend_addr).await.is_ok()
    }

    async fn shutdown_gracefully(&self) {
        loop {
            let total_active_connections: u64 = self
                .inner
                .config
                .backend_addresses
                .iter()
                .map(|backend| backend.active_connections.load(Ordering::Relaxed))
                .sum();

            if total_active_connections == 0 {
                return;
            }

            tracing::info!(
                active_connections = total_active_connections,
                "Waiting for connections to close"
            );

            tokio::time::sleep(Duration::from_millis(500)).await
        }
    }
}
