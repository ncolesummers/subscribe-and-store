// src/mqtt.rs
use anyhow::Result;
use paho_mqtt::{
    AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Message, SubscribeOptions, MQTT_VERSION_5, QOS_1,
};
use std::time::Duration;
use tokio::time as tokio_time;

pub async fn connect(mqtt_uri: &str, client_id: &str) -> AsyncClient {
    let create_opts = CreateOptionsBuilder::new()
        .server_uri(mqtt_uri)
        .client_id(client_id)
        .finalize();

    let client = AsyncClient::new(create_opts).expect("Failed to create MQTT client");

    let conn_opts = ConnectOptionsBuilder::with_mqtt_version(MQTT_VERSION_5)
        .keep_alive_interval(Duration::from_secs(20))
        .finalize();

    // Retry logic with error handling and waiting
    loop {
        match client.connect(conn_opts.clone()).await {
            Ok(_) => {
                println!("Successfully connected to MQTT broker at {}", mqtt_uri);
                break; // Successfully connected, exit the function
            }
            Err(e) => {
                // Use anyhow to add context to the error before logging it
                let error = anyhow::Error::new(e)
                    .context("Failed to connect to MQTT broker, retrying in 5 seconds...");
                println!("{}", error);

                tokio_time::sleep(Duration::from_secs(5)).await;
            }
        }
    }

    client
}

pub async fn subscribe(client: &AsyncClient, topics: &[&str], qos: &[i32]) -> Result<()> {
    println!("Subscribing to topics: {:?}", topics);
    let sub_opts = vec![SubscribeOptions::with_retain_as_published(); topics.len()];

    let mut attempts = 0;
    loop {
        attempts += 1;
        match client
            .subscribe_many_with_options(topics, qos, &sub_opts, None)
            .await
        {
            Ok(_) => {
                println!("Successfully subscribed to topics: {:?}", topics);
                return Ok(());
            }
            Err(e) => {
                eprintln!(
                    "Attempt {}: Error subscribing to topics: {:?}, error: {}",
                    attempts, topics, e
                );
                if attempts >= 5 {
                    // Suppose we want to limit the number of attempts to 5
                    return Err(anyhow::Error::new(e).context(
                        "Exceeded maximum retry attempts for subscribing to MQTT topics",
                    ));
                }
                // Wait for a bit before retrying. Adjust the duration as needed.
                tokio_time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn publish(client: AsyncClient, message: Vec<u8>, topic: &str) {
    let msg = Message::new(topic, message, QOS_1);
    if let Err(e) = client.publish(msg).await {
        println!("Error publishing the message: {:?}", e);
    }
}