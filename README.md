# Subscribe and Store - MQTT to TimescaleDB Service

A high-performance Rust service that subscribes to MQTT topics and stores robotic arm telemetry data in a TimescaleDB time-series database. Built for real-time data ingestion with automatic reconnection and error handling.

## Overview

This service is designed to handle high-frequency telemetry data from robotic arms, subscribing to MQTT topics (`left_arm` and `right_arm`) and persisting the data to TimescaleDB for time-series analysis. The system processes transformation matrices and joint data from multiple robotic components.

## Features

- **Real-time MQTT Subscription**: Subscribes to multiple topics with configurable QoS levels
- **TimescaleDB Integration**: Efficient time-series data storage with hypertables
- **Automatic Reconnection**: Robust error handling with automatic MQTT reconnection
- **High Performance**: Built with Rust and Tokio for asynchronous I/O
- **JSON Data Processing**: Handles complex robotic arm data structures with multiple joint and finger matrices
- **Environment Configuration**: Flexible configuration through environment variables

## Architecture

```
┌─────────────┐      MQTT       ┌──────────────┐      SQL        ┌──────────────┐
│ MQTT Broker │ ───────────────> │ Rust Service │ ───────────────> │ TimescaleDB  │
│             │   (left_arm,     │              │   (JSONB data)   │              │
│             │    right_arm)    │              │                  │              │
└─────────────┘                  └──────────────┘                  └──────────────┘
```

## Data Model

### ArmMessage Structure
```rust
{
  timestamp: i64,              // Unix timestamp in milliseconds
  matrices: {
    J1-J9: Vec<Vec<f64>>,     // Joint transformation matrices (9 joints)
    F1-F6: Vec<Vec<f64>>      // Finger transformation matrices (6 fingers)
  }
}
```

### Database Schema
- **Tables**: `left_arm`, `right_arm`
- **Columns**:
  - `timestamp` (TIMESTAMPTZ): Time of data capture
  - `data` (JSONB): Complete arm telemetry data
- **Optimization**: TimescaleDB hypertables for efficient time-series queries

## Prerequisites

- Rust 1.70+ (2021 edition)
- Docker and Docker Compose
- PostgreSQL client libraries

## Installation

### 1. Clone the Repository
```bash
git clone <repository-url>
cd subscribe-and-store
```

### 2. Set Up TimescaleDB
```bash
cd db
docker-compose up -d
```

This will:
- Start a TimescaleDB instance on port 5432
- Create the database `sabino-ts`
- Initialize hypertables for `left_arm` and `right_arm`

### 3. Configure Environment
Create a `.env` file in the project root:

```env
# Database Configuration
DATABASE_URL=postgres://postgres:sabino@localhost:5432/sabino-ts

# MQTT Configuration (optional, defaults shown)
RUST_SUB_MQTT_URI=tcp://localhost:1883
RUST_SUB_MQTT_CLIENT_ID=rust_sub_mqtt_tsdb_client
```

### 4. Build and Run
```bash
# Development
cargo run

# Production (optimized)
cargo build --release
./target/release/subscribe-and-store
```

## Configuration

| Environment Variable | Description | Default |
|---------------------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `RUST_SUB_MQTT_URI` | MQTT broker URI | `tcp://localhost:1883` |
| `RUST_SUB_MQTT_CLIENT_ID` | MQTT client identifier | `rust_sub_mqtt_tsdb_client` |

## Dependencies

### Core Dependencies
- **paho-mqtt** (0.12): MQTT client with bundled C library
- **tokio** (1.x): Async runtime with full features
- **sqlx** (0.7.3): Async PostgreSQL driver with compile-time query verification
- **serde** (1.0): JSON serialization/deserialization
- **anyhow** (1.0): Error handling with context

### Database
- **TimescaleDB**: PostgreSQL extension for time-series data
- **PostgreSQL 16**: Latest stable release

## Development

### Project Structure
```
subscribe-and-store/
├── src/
│   ├── main.rs      # Application entry point and message loop
│   ├── mqtt.rs      # MQTT connection and subscription logic
│   ├── sql.rs       # Database operations and connection pooling
│   └── model.rs     # Data structures for arm messages
├── db/
│   ├── docker-compose.yml  # TimescaleDB container configuration
│   └── init.sql            # Database initialization script
├── Cargo.toml              # Rust dependencies
└── README.md
```

### Testing
```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Performance Considerations
- **QoS 0**: Used for maximum throughput (at-most-once delivery)
- **Connection Pooling**: 5 database connections for concurrent writes
- **Stream Buffer**: 25 messages buffered for processing
- **Keep-Alive**: 20-second interval for connection monitoring

## Monitoring

The service provides console output for:
- Connection status (MQTT and database)
- Subscription confirmations
- Reconnection attempts
- Error messages with context

## Error Handling

- **MQTT Disconnection**: Automatic reconnection with exponential backoff
- **Database Errors**: Logged with context, continues processing other messages
- **Parse Errors**: Detailed error messages with problematic data
- **Subscription Failures**: Retry up to 5 times with 5-second intervals

## Docker Deployment

### Building the Service Container
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates
COPY --from=builder /app/target/release/subscribe-and-store /usr/local/bin/
CMD ["subscribe-and-store"]
```

### Docker Compose Full Stack
```yaml
version: "3.8"
services:
  app:
    build: .
    environment:
      - DATABASE_URL=postgres://postgres:sabino@timescaledb:5432/sabino-ts
      - RUST_SUB_MQTT_URI=tcp://mqtt:1883
    depends_on:
      - timescaledb
    restart: unless-stopped

  timescaledb:
    image: timescale/timescaledb:latest-pg16
    # ... (existing configuration)
```

## Troubleshooting

### Common Issues

1. **Connection Refused to Database**
   - Verify TimescaleDB is running: `docker ps`
   - Check DATABASE_URL in `.env`
   - Ensure PostgreSQL port 5432 is not blocked

2. **MQTT Connection Failed**
   - Verify MQTT broker is running
   - Check RUST_SUB_MQTT_URI configuration
   - Ensure network connectivity to broker

3. **Parse Errors**
   - Verify incoming JSON structure matches ArmMessage format
   - Check timestamp is in milliseconds
   - Ensure all joint and finger matrices are present

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here]

## Support

For issues and questions, please [create an issue](link-to-issues) in the repository.