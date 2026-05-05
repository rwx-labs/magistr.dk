# magistr.dk

A high-performance quote management and display service.

## Tech Stack

- **Rust**
- **Axum** (Web Framework)
- **SQLx** (PostgreSQL)
- **Askama** (Templating)
- **OpenTelemetry** & **Jaeger** (Observability)

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/)
- [Docker](https://www.docker.com/) or [Podman](https://podman.io/)

### Local Development

1. **Start dependencies**:
   ```bash
   docker compose up -d
   ```

2. **Run the application**:
   ```bash
   cargo run
   ```
   *Note: Database migrations are applied automatically on startup.*

## Configuration

Environment variables (prefixed with `MAGISTR_`):

| Variable | Default | Description |
|---|---|---|
| `MAGISTR_DATABASE_URL` | `postgresql://magistr:password@localhost/magistr_development` | Database connection URL |
| `MAGISTR_HTTP_ADDRESS` | `[::]:3000` | HTTP server address |
| `MAGISTR_TRACING_ENABLED` | `true` | Enable OpenTelemetry tracing |
| `MAGISTR_DATABASE_MAX_CONNECTIONS` | `5` | Max DB connection pool size |
| `MAGISTR_DATABASE_IDLE_TIMEOUT` | `30` | DB connection idle timeout (seconds) |
| `MAGISTR_HTTP_COMPRESSION` | `true` | Enable HTTP response compression |
