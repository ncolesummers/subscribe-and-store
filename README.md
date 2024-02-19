# Sabino Subscribe and Store

This is a simple service that will subscribe to the left_arm and right_arm MQTT topics and store that data in a time series database (TimeScaleDB).

## Setup

1. Install Docker and Docker Compose
2. Install Rust
3. Write a `.env` file with the following content:

```
TIMESCALE_DATABASE_URL=postgres://postgres:password@localhost:5432/postgres
```