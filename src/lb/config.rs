use crate::lb::{
    backend::Backend,
    strategies::{least_connections::LeastConnections, round_robin::RoundRobin},
    strategy::SelectionStrategy,
};
use anyhow::{Context, Result, bail};
use std::{
    env,
    net::SocketAddr,
    sync::atomic::{AtomicBool, AtomicU64},
    time::Duration,
};

const ADDR_ENV_VAR_NAME: &str = "OXIDE_LB_BIND_ADDR";
const BACKEND_ADDRESSES_ENV_VAR_NAME: &str = "OXIDE_LB_BACKEND_ADDRESSES";
const HEALTH_CHECK_INTERVAL_ENV_VAR_NAME: &str = "OXIDE_LB_HEALTH_CHECK_INTERVAL";
const SELECTION_STRATEGY_ENV_VAR_NAME: &str = "OXIDE_LB_SELECTION_STRATEGY";

const ROUND_ROBIN_STRATEGY_NAME: &str = "ROUND_ROBIN";
const LEAST_CONNECTIONS_STRATEGY_NAME: &str = "LEAST_CONNECTIONS";

pub struct Config {
    pub bind_addr: SocketAddr,
    pub backend_addresses: Vec<Backend>,
    pub health_check_interval: Duration,
    pub strategy: Box<dyn SelectionStrategy>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let addr_str = Self::get_env_var(ADDR_ENV_VAR_NAME)?;

        let bind_addr = addr_str.parse::<SocketAddr>().context(format!(
            "Invalid address format in {ADDR_ENV_VAR_NAME}='{addr_str}'"
        ))?;

        let backend_addresses_str = Self::get_env_var(BACKEND_ADDRESSES_ENV_VAR_NAME)?;

        let addresses = Self::parse_backend_addresses(&backend_addresses_str)?;

        if addresses.is_empty() {
            bail!("No backend addresses provided");
        }

        let health_check_interval_str = Self::get_env_var(HEALTH_CHECK_INTERVAL_ENV_VAR_NAME)?;

        let health_check_interval = health_check_interval_str.parse::<u64>().context(format!(
            "Invalid health check interval format in {HEALTH_CHECK_INTERVAL_ENV_VAR_NAME}='{health_check_interval_str}'"
        ))?;

        if health_check_interval == 0 {
            bail!("Health check interval must be greater than 0 seconds");
        }

        let selection_strategy_str = Self::get_env_var(SELECTION_STRATEGY_ENV_VAR_NAME)?;

        let strategy: Box<dyn SelectionStrategy> = match selection_strategy_str.as_str() {
            ROUND_ROBIN_STRATEGY_NAME => Box::new(RoundRobin::new()),
            LEAST_CONNECTIONS_STRATEGY_NAME => Box::new(LeastConnections::new()),
            _ => {
                bail!(
                    "Unknown selection strategy. Available options: {ROUND_ROBIN_STRATEGY_NAME}, {LEAST_CONNECTIONS_STRATEGY_NAME}."
                )
            }
        };

        Ok(Self {
            bind_addr,
            backend_addresses: addresses,
            health_check_interval: Duration::from_secs(health_check_interval),
            strategy,
        })
    }

    fn parse_backend_addresses(backend_addresses_str: &str) -> Result<Vec<Backend>> {
        backend_addresses_str
            .split(",")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let addr = s.parse::<SocketAddr>().context(format!(
                    "Invalid address '{s}' in {BACKEND_ADDRESSES_ENV_VAR_NAME}"
                ))?;

                Ok(Backend {
                    addr,
                    is_alive: AtomicBool::new(true),
                    active_connections: AtomicU64::new(0),
                })
            })
            .collect()
    }

    fn get_env_var(key: &str) -> Result<String> {
        env::var(key).context(format!(
            "Environment variable {key} is missing or contains invalid unicode"
        ))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::*;

    #[test]
    fn all_backend_addresses_should_be_alive_by_default() {
        let input = "93.184.216.34:80, 192.168.1.112:8080, 237.211.242.234:3000";
        let result = Config::parse_backend_addresses(input).unwrap();

        assert!(result.iter().all(|v| v.is_alive.load(Ordering::Relaxed)));
    }

    #[test]
    fn all_backend_addresses_should_have_no_active_connections_by_default() {
        let input = "93.184.216.34:80, 192.168.1.112:8080, 237.211.242.234:3000";
        let result = Config::parse_backend_addresses(input).unwrap();

        assert!(
            result
                .iter()
                .all(|v| v.active_connections.load(Ordering::Relaxed) == 0)
        );
    }

    #[test]
    fn should_parse_valid_backend_addresses() {
        let input = "93.184.216.34:80, 192.168.1.112:8080, 237.211.242.234:3000";
        let result = Config::parse_backend_addresses(input).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].addr, SocketAddr::from(([93, 184, 216, 34], 80)));
        assert_eq!(result[1].addr, SocketAddr::from(([192, 168, 1, 112], 8080)));
        assert_eq!(
            result[2].addr,
            SocketAddr::from(([237, 211, 242, 234], 3000))
        );
    }

    #[test]
    fn should_parse_backend_addresses_with_different_spaces() {
        let input = "   93.184.216.34:80,192.168.1.112:8080,        237.211.242.234:3000    ";
        let result = Config::parse_backend_addresses(input).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].addr, SocketAddr::from(([93, 184, 216, 34], 80)));
        assert_eq!(result[1].addr, SocketAddr::from(([192, 168, 1, 112], 8080)));
        assert_eq!(
            result[2].addr,
            SocketAddr::from(([237, 211, 242, 234], 3000))
        );
    }

    #[test]
    fn should_return_error_for_invalid_address() {
        let input = "999.999.999.999:80, 93.184.216.34:80";
        let result = Config::parse_backend_addresses(input);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid address"));
    }

    #[test]
    fn should_return_empty_vec_for_empty_input() {
        let input = "";
        let result = Config::parse_backend_addresses(input).unwrap();

        assert!(result.is_empty());
    }
}
