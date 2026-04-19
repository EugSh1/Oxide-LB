use anyhow::Result;
use oxide_lb::lb::{config::Config, load_balancer::LoadBalancer};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(env_filter)
        .init();

    if let Err(error) = run().await {
        tracing::error!(
            error = %error,
            details = ?error,
            "Application terminated with error"
        );
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let config = Config::from_env()?;
    let lb = LoadBalancer::new(config);

    lb.run().await?;

    Ok(())
}
