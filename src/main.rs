use crate::sql::{init_pg_pool, write_data_to_db};
use anyhow::Context;
use futures::StreamExt;
use mqtt::{connect, subscribe};
use std::time::Duration;

mod model;
mod mqtt;
mod sql;

// Constants for MQTT topics and QoS levels
const TOPICS: &[&str] = &["left_arm", "right_arm"];
// QoS 0 for at most once delivery.
// Lower QoS levels are faster and have less overhead, which
// Since this is a hot path, we use QoS 0 for the fastest delivery.
const QOS: &[i32] = &[0, 0];

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mqtt_uri =
        &dotenv::var("RUST_SUB_MQTT_URI").unwrap_or_else(|_| "tcp://localhost:1883".to_string());
    let client_id = &dotenv::var("RUST_SUB_MQTT_CLIENT_ID")
        .unwrap_or_else(|_| "rust_sub_mqtt_tsdb_client".to_string());
    let pool = init_pg_pool().context("Failed to create pool").unwrap();

    let mut client = connect(mqtt_uri, client_id).await;
    let mut stream = client.get_stream(25);
    subscribe(&client, TOPICS, QOS)
        .await
        .context("Failed to complete subscription")?;

    // Inside the async block where messages are processed
    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            // println!("Received message: {}", msg);
            if let Err(e) = write_data_to_db(&msg.payload_str(), msg.topic(), &pool).await {
                eprintln!("Failed to write data to DB: {}", e);
            }
        } else {
            println!("Lost connection. Attempting reconnect.");
            while let Err(err) = client.reconnect().await {
                println!("Error reconnecting: {}", err);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    Ok(())
}
