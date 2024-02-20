use futures::stream::StreamExt;
use once_cell::sync::Lazy;
use paho_mqtt::{self as mqtt, MQTT_VERSION_5};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration as StdDuration;
use time::{Duration, OffsetDateTime};
use tokio::time as tokio_time;

const TOPICS: &[&str] = &["left_arm", "right_arm"];

// QoS 0 for at most once delivery.
// Lower QoS levels are faster and have less overhead, which
// Since this is a hot path, we use QoS 0 for the fastest delivery.
const QOS: &[i32] = &[0, 0];

static POOL: Lazy<Pool<Postgres>> = Lazy::new(|| {
    let database_url = &dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5) // Example: set maximum connections to 5
        .connect_lazy(database_url)
        .expect("Failed to create pool")
});

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ArmMessage {
    pub timestamp: i64,
    pub matrices: Arm,
}

// 9 joints j1-j9 and 6 fingers f1-f6
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Arm {
    pub J1: Vec<Vec<f64>>,
    pub J2: Vec<Vec<f64>>,
    pub J3: Vec<Vec<f64>>,
    pub J4: Vec<Vec<f64>>,
    pub J5: Vec<Vec<f64>>,
    pub J6: Vec<Vec<f64>>,
    pub J7: Vec<Vec<f64>>,
    pub J8: Vec<Vec<f64>>,
    pub J9: Vec<Vec<f64>>,
    pub F1: Vec<Vec<f64>>,
    pub F2: Vec<Vec<f64>>,
    pub F3: Vec<Vec<f64>>,
    pub F4: Vec<Vec<f64>>,
    pub F5: Vec<Vec<f64>>,
    pub F6: Vec<Vec<f64>>,
}

async fn write_data_to_db(data: &str, table: &str) -> Result<(), sqlx::Error> {
    // Data is a JSON string, so you can parse it into a struct using serde_json
    let arm: ArmMessage = match serde_json::from_str(data) {
        Ok(arm) => arm,
        Err(e) => {
            return Err(sqlx::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e,
            )))
        }
    };

    insert_data(table, arm, &POOL).await?;

    Ok(())
}

async fn insert_data(
    table: &str,
    arm: ArmMessage,
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    // Parse the epoch timestamp from the ArmMessage, assuming it's in milliseconds
    let epoch_milliseconds = arm.timestamp;

    // Convert milliseconds to seconds and nanoseconds
    let seconds = epoch_milliseconds / 1000;
    let nanoseconds = (epoch_milliseconds % 1000) * 1_000_000; // Convert remainder to nanoseconds

    // Create OffsetDateTime from seconds and nanoseconds
    let parsed_timestamp = OffsetDateTime::from_unix_timestamp(seconds)
        .map(|dt| dt + Duration::nanoseconds(nanoseconds))
        .map_err(|_| {
            sqlx::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Timestamp out of range",
            ))
        })?;

    // Serialize the matrices to a JSON string
    let matrices_json = serde_json::to_string(&arm.matrices)
        .map_err(|e| sqlx::Error::from(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

    match table {
        "left_arm" | "right_arm" => {
            let query = format!(
                "INSERT INTO {} (timestamp, data) VALUES ($1, $2::jsonb)",
                table
            );
            sqlx::query(&query)
                .bind(parsed_timestamp) // Bind the converted DateTime<Utc> here
                .bind(&matrices_json) // This is already correctly serialized to JSON
                .execute(pool)
                .await?;
        }
        _ => {
            return Err(sqlx::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid table name",
            )))
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    // Read environment variables
    let mqtt_uri =
        &dotenv::var("RUST_SUB_MQTT_URI").unwrap_or_else(|_| "tcp://localhost:1883".to_string());
    let client_id = &dotenv::var("RUST_SUB_MQTT_CLIENT_ID")
        .unwrap_or_else(|_| "rust_sub_mqtt_tsdb_client".to_string());

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(mqtt_uri)
        .client_id(client_id)
        .finalize();

    let mut client = mqtt::AsyncClient::new(create_opts).unwrap();

    let mut stream = client.get_stream(25);

    let conn_opts = mqtt::ConnectOptionsBuilder::with_mqtt_version(MQTT_VERSION_5)
        .keep_alive_interval(StdDuration::from_secs(20))
        .finalize();

    // Retry connecting to the MQTT broker until successful
    loop {
        match client.connect(conn_opts.clone()).await {
            Ok(_) => {
                println!("Successfully connected to MQTT broker at {}", mqtt_uri);
                break;
            }
            Err(e) => {
                println!(
                    "Failed to connect to MQTT broker: {:?}, retrying in 5 seconds...",
                    e
                );
                tokio_time::sleep(StdDuration::from_secs(5)).await;
            }
        }
    }

    println!("Subscribing to topics: {:?}", TOPICS);
    let sub_opts = vec![mqtt::SubscribeOptions::with_retain_as_published(); TOPICS.len()];
    // Retry logic with error handling and waiting
    loop {
        match client
            .subscribe_many_with_options(TOPICS, QOS, &sub_opts, None)
            .await
        {
            Ok(_) => {
                println!("Successfully subscribed to topics: {:?}", TOPICS);
                break; // Exit loop on successful subscription
            }
            Err(e) => {
                eprintln!("Error subscribing to topics: {:?}, error: {}", TOPICS, e);
                // Wait for a bit before retrying. Adjust the duration as needed.
                tokio_time::sleep(StdDuration::from_secs(5)).await;
            }
        }
    }

    // Inside the async block where messages are processed
    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            // println!("Received message: {}", msg);
            if let Err(e) = write_data_to_db(&msg.payload_str(), msg.topic()).await {
                eprintln!("Failed to write data to DB: {}", e);
            }
        } else {
            println!("Lost connection. Attempting reconnect.");
            while let Err(err) = client.reconnect().await {
                println!("Error reconnecting: {}", err);
                tokio::time::sleep(StdDuration::from_secs(1)).await;
            }
        }
    }
}
