CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

-- Creating the left_arm table
CREATE TABLE IF NOT EXISTS left_arm (
    timestamp TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL
);

-- Convert the left_arm table into a hypertable
SELECT create_hypertable('left_arm', 'timestamp');

-- Creating the right_arm table
CREATE TABLE IF NOT EXISTS right_arm (
    timestamp TIMESTAMPTZ NOT NULL,
    data JSONB NOT NULL
);

-- Convert the right_arm table into a hypertable
SELECT create_hypertable('right_arm', 'timestamp');
