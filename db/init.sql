CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

-- Creating the left_arm table
CREATE TABLE left_arm (
    timestamp TIMESTAMPTZ NOT NULL,
    j1 JSON NOT NULL,
    j2 JSON NOT NULL,
    j3 JSON NOT NULL,
    j4 JSON NOT NULL,
    j5 JSON NOT NULL,
    j6 JSON NOT NULL,
    j7 JSON NOT NULL,
    j8 JSON NOT NULL,
    j9 JSON NOT NULL,
    f1 JSON NOT NULL,
    f2 JSON NOT NULL,
    f3 JSON NOT NULL,
    f4 JSON NOT NULL,
    f5 JSON NOT NULL,
    f6 JSON NOT NULL
);

-- Convert the left_arm table into a hypertable
SELECT create_hypertable('left_arm', 'timestamp');

-- Creating the right_arm table
CREATE TABLE right_arm (
    timestamp TIMESTAMPTZ NOT NULL,
    j1 JSON NOT NULL,
    j2 JSON NOT NULL,
    j3 JSON NOT NULL,
    j4 JSON NOT NULL,
    j5 JSON NOT NULL,
    j6 JSON NOT NULL,
    j7 JSON NOT NULL,
    j8 JSON NOT NULL,
    j9 JSON NOT NULL,
    f1 JSON NOT NULL,
    f2 JSON NOT NULL,
    f3 JSON NOT NULL,
    f4 JSON NOT NULL,
    f5 JSON NOT NULL,
    f6 JSON NOT NULL
);

-- Convert the right_arm table into a hypertable
SELECT create_hypertable('right_arm', 'timestamp');
