// src/mqtt.rs
use paho_mqtt::{self as mqtt, MQTT_VERSION_5, AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder};
use std::time::Duration as StdDuration;
use tokio::time as tokio_time;

pub async fn connect(mqtt_uri: &str, client_id: &str) -> mqtt::AsyncClient {
    let create_opts = CreateOptionsBuilder::new()
        .server_uri(mqtt_uri)
        .client_id(client_id)
        .finalize();

    let client = mqtt::AsyncClient::new(create_opts).expect("Failed to create MQTT client");

    let conn_opts = ConnectOptionsBuilder::with_mqtt_version(MQTT_VERSION_5)
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

    client
}

pub async fn subscribe(client: &mqtt::AsyncClient, topics: &[&str], qos: &[i32]) {
    println!("Subscribing to topics: {:?}", topics);
    let sub_opts = vec![mqtt::SubscribeOptions::with_retain_as_published(); topics.len()];
    // Retry logic with error handling and waiting
    loop {
        match client
            .subscribe_many_with_options(topics, qos, &sub_opts, None)
            .await
        {
            Ok(_) => {
                println!("Successfully subscribed to topics: {:?}", topics);
                break; // Exit loop on successful subscription
            }
            Err(e) => {
                eprintln!("Error subscribing to topics: {:?}, error: {}", topics, e);
                // Wait for a bit before retrying. Adjust the duration as needed.
                tokio_time::sleep(StdDuration::from_secs(5)).await;
            }
        }
    }
}
