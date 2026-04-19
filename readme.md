# Oxide-LB

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.94-orange.svg)](https://www.rust-lang.org)
[![CI](https://github.com/EugSh1/oxide-lb/actions/workflows/ci.yml/badge.svg)](https://github.com/EugSh1/oxide-lb/actions/workflows/ci.yml)
[![Build & Publish](https://github.com/EugSh1/oxide-lb/actions/workflows/build.yml/badge.svg)](https://github.com/EugSh1/oxide-lb/actions/workflows/build.yml)

A **fast, lightweight, and safe L4 (TCP) load balancer** built with Rust and Tokio. Oxide-LB is
designed to be a highly concurrent, low-overhead reverse proxy that efficiently distributes TCP
traffic across multiple backend servers.

## Features

- **High-Performance L4 Routing** ⚡️
    - Proxies raw TCP connections asynchronously using zero-cost abstractions.
    - Minimal resource footprint thanks to Rust and Tokio.
- **Multiple Routing Strategies** 🔀
    - `ROUND_ROBIN`: Distributes requests sequentially.
    - `LEAST_CONNECTIONS`: Smartly routes traffic to the backend with the fewest active connections.
- **Active Health Checking** ⏱️
    - A background worker continuously monitors backend availability.
    - Automatically removes dead backends' addresses from the pool and seamlessly restores them when
      they recover.
- **Graceful Shutdown** 🛑
    - Listens for `SIGTERM` and `Ctrl+C`.
    - Waits for all active connections to finish before terminating, ensuring zero dropped requests
      during deployments.
- **Structured Observability** 📊
    - Fully integrated JSON-based structured logging using `tracing`.
    - Easy to parse by log aggregators.
- **Secure by Default (Docker)** 🐳
    - Provided Docker image uses a multi-stage build and runs the application as a non-root,
      unprivileged user.

## Technologies Used

**Core & Networking:**

- **Rust (2024 Edition)**: Memory safety, fearless concurrency, and performance.
- **Tokio**: Industry-standard asynchronous runtime for high-performance I/O.

**Observability & Utilities:**

- **Tracing / Tracing-Subscriber**: For structured logging.
- **Anyhow**: For flexible error handling.

**DevOps:**

- **Docker**: Containerization and deployment.
- **GitHub Actions**: Automated CI/CD pipeline.

![Made with](https://go-skill-icons.vercel.app/api/icons?i=rust,tokiors,docker&theme=dark)

## Configuration

Oxide-LB is entirely configured via environment variables, making it perfect for Docker and
Kubernetes environments.

| Environment Variable             | Default        | Description                                                                      |
| :------------------------------- | :------------- | :------------------------------------------------------------------------------- |
| `OXIDE_LB_BIND_ADDR`             | `0.0.0.0:3000` | The address and port the load balancer will listen on.                           |
| `OXIDE_LB_BACKEND_ADDRESSES`     | _(Required)_   | Comma-separated list of backend TCP addresses (e.g., `10.0.0.1:80,10.0.0.2:80`). |
| `OXIDE_LB_HEALTH_CHECK_INTERVAL` | `5`            | Time in seconds between backend health checks.                                   |
| `OXIDE_LB_SELECTION_STRATEGY`    | `ROUND_ROBIN`  | Routing algorithm. Options: `ROUND_ROBIN`, `LEAST_CONNECTIONS`.                  |
| `RUST_LOG`                       | `info`         | Log level for the tracing subscriber (e.g., `debug`, `info`, `warn`).            |

## Quick Start with Docker Compose

The easiest way to see Oxide-LB in action is to use the provided `compose.yml`. This setup launches
the load balancer and three sample "whoami" backend services.

1. **Start the demo:**

    ```bash
    make demo
    ```

    or

    ```bash
    docker compose up --build
    ```

2. **Test the load balancing:** Since Oxide-LB works at the TCP level, you can use `curl` to send
   HTTP requests through it. Run this command multiple times:

    ```bash
    curl http://localhost:3000
    ```

    **Example Output:** You will notice the `Hostname` changing with each request, proving that
    Oxide-LB is cycling through the backends:

    ```text
    Hostname: 59da77461623
    ...
    Hostname: a1b2c3d4e5f6
    ```

3. **Test Health Checks:** Stop one of the backend containers:
    ```bash
    docker compose stop app-1
    ```
    Oxide-LB will detect the failure within 5 seconds and stop routing traffic to it. Once you
    `docker compose start app-1`, it will automatically be brought back into the pool.

## Usage

### 1. Running with Docker (Recommended)

You can easily run the load balancer using Docker. Just pass the required environment variables:

```bash
docker run -d \
  -p 3000:3000 \
  -e OXIDE_LB_BIND_ADDR=0.0.0.0:3000 \
  -e OXIDE_LB_BACKEND_ADDRESSES="192.168.1.10:8080,192.168.1.11:8080" \
  -e OXIDE_LB_SELECTION_STRATEGY="LEAST_CONNECTIONS" \
  -e OXIDE_LB_HEALTH_CHECK_INTERVAL=5 \
  --name oxide-lb \
  ghcr.io/eugsh1/oxide-lb:latest
```

### 2. Running Locally (Development)

**Prerequisites:**

- Rust toolchain (1.94+ recommended)

1. Clone the repository:
    ```bash
    git clone https://github.com/EugSh1/oxide-lb.git
    cd oxide-lb
    ```
2. Export variables and run:

    ```bash
    export OXIDE_LB_BIND_ADDR="127.0.0.1:3000"
    export OXIDE_LB_BACKEND_ADDRESSES="127.0.0.1:8081, 127.0.0.1:8082"
    export OXIDE_LB_HEALTH_CHECK_INTERVAL=5
    export RUST_LOG="debug"

    cargo run --release
    ```

## Architecture

1. **Accept Loop**: The main thread listens for incoming TCP connections.
2. **Strategy Selection**: Upon connection, the router selects the optimal backend based on the
   configured strategy (`LeastConnections` / `RoundRobin`).
3. **Connection Guard**: An atomic counter increments to track active connections per backend.
4. **Bi-directional Stream**: Traffic is piped bidirectionally between the client and the backend
   using `tokio::io::copy_bidirectional`.
5. **Cleanup**: When the socket closes, the `ConnectionGuard` automatically drops, decrementing the
   active connection counter.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
