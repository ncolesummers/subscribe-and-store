use paho_mqtt as mqtt;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use tokio::time;
use futures::stream::StreamExt;
use serde_json;

const TOPICS: &[&str] = &["left_arm", "right_arm"];
const QOS: &[i32] = &[1, 1];

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ArmMessage {
    pub timestamp: String,
    pub matrices: Arm,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FourByFourMatrix {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub w: Vec<f64>,
}

// 9 joints j1-j9 and 6 fingers f1-f6
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Arm {
    pub j1: FourByFourMatrix,
    pub j2: FourByFourMatrix,
    pub j3: FourByFourMatrix,
    pub j4: FourByFourMatrix,
    pub j5: FourByFourMatrix,
    pub j6: FourByFourMatrix,
    pub j7: FourByFourMatrix,
    pub j8: FourByFourMatrix,
    pub j9: FourByFourMatrix,
    pub f1: FourByFourMatrix,
    pub f2: FourByFourMatrix,
    pub f3: FourByFourMatrix,
    pub f4: FourByFourMatrix,
    pub f5: FourByFourMatrix,
    pub f6: FourByFourMatrix,
}

async fn write_data_to_db(data: &str, table: &str) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .connect(&dotenv::var("TIMESCALE_DATABASE_URL").unwrap())
        .await?;

    // Data is a JSON string, so you can parse it into a struct using serde_json
    let arm: ArmMessage = serde_json::from_str(data)?;
    
    sqlx::query!("INSERT INTO your_table_name (time, data) VALUES (NOW(), $1)", data)
        .execute(&pool)
        .await?;

    Ok(())
}


#[tokio::main]
async fn main() {
    // Read environment variables
    let mqtt_uri = &dotenv::var("RUST_SUB_MQTT_URI").unwrap_or_else(|_| "tcp://localhost:1883".to_string());
    let client_id = &dotenv::var("RUST_SUB_MQTT_CLIENT_ID").unwrap_or_else(|_| "rust_sub_mqtt_tsdb_client".to_string());

    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(mqtt_uri)
        .client_id(client_id)
        .finalize();

    let mut client = mqtt::AsyncClient::new(create_opts).unwrap();

    let mut stream = client.get_stream(25);

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Retry connecting to the MQTT broker until successful
    loop {
        match client.connect(conn_opts.clone()).await {
            Ok(_) => {
                println!("Successfully connected to MQTT broker at {}", mqtt_uri);
                break;
            },
            Err(e) => {
                println!("Failed to connect to MQTT broker: {:?}, retrying in 5 seconds...", e);
                time::sleep(time::Duration::from_secs(5)).await;
            }
        }
    }

    println!("Subscribing to topics: {:?}", TOPICS);
    let sub_opts = vec![mqtt::SubscribeOptions::with_retain_as_published(); TOPICS.len()];
    // Retry logic with error handling and waiting
    loop {
        match client.subscribe_many_with_options(TOPICS, QOS, &sub_opts, None).await {
            Ok(_) => {
                println!("Successfully subscribed to topics: {:?}", TOPICS);
                break; // Exit loop on successful subscription
            },
            Err(e) => {
                eprintln!("Error subscribing to topics: {:?}, error: {}", TOPICS, e);
                // Wait for a bit before retrying. Adjust the duration as needed.
                time::sleep(Duration::from_secs(5)).await;
            }
        }
    }


    // Inside the async block where messages are processed
    while let Some(msg_opt) = stream.next().await {
        if let Some(msg) = msg_opt {
            println!("Received message: {}", msg);
            if let Err(e) = write_data_to_db(&msg.payload_str()).await {
                eprintln!("Failed to write data to DB: {}", e);
            }
        }
        else {
            println!("Lost connection. Attempting reconnect.");
            while let Err(err) = client.reconnect().await {
                println!("Error reconnecting: {}", err);
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

}
