use paho_mqtt as mqtt;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use tokio::time;
use futures::stream::StreamExt;
use serde_json;
use sqlx::Postgres;

const TOPICS: &[&str] = &["left_arm", "right_arm"];
const QOS: &[i32] = &[1, 1];

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ArmMessage {
    pub timestamp: String,
    pub matrices: Arm,
}

// 9 joints j1-j9 and 6 fingers f1-f6
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Arm {
    pub j1: Vec<Vec<f64>>,
    pub j2: Vec<Vec<f64>>,
    pub j3: Vec<Vec<f64>>,
    pub j4: Vec<Vec<f64>>,
    pub j5: Vec<Vec<f64>>,
    pub j6: Vec<Vec<f64>>,
    pub j7: Vec<Vec<f64>>,
    pub j8: Vec<Vec<f64>>,
    pub j9: Vec<Vec<f64>>,
    pub f1: Vec<Vec<f64>>,
    pub f2: Vec<Vec<f64>>,
    pub f3: Vec<Vec<f64>>,
    pub f4: Vec<Vec<f64>>,
    pub f5: Vec<Vec<f64>>,
    pub f6: Vec<Vec<f64>>,
}

async fn write_data_to_db(data: &str, table: &str) -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .connect(&dotenv::var("DATABASE_URL").unwrap())
        .await?;


    // Data is a JSON string, so you can parse it into a struct using serde_json
    let arm: ArmMessage = match serde_json::from_str(data) {
        Ok(arm) => arm,
        Err(e) => return Err(sqlx::Error::from(std::io::Error::new(std::io::ErrorKind::InvalidData, e))),
    };

    insert_data(table, arm, &pool).await?;

    Ok(())
}

async fn insert_data(table: &str, arm: ArmMessage, pool: &sqlx::Pool<Postgres>) -> Result<(), sqlx::Error> {
    match table {
        "left_arm" => {
            sqlx::query!("INSERT INTO left_arm (timestamp, data) VALUES ($1, $2)", arm.timestamp, arm.matrices)
                .execute(pool)
                .await?;
        },
        "right_arm" => {
            sqlx::query!("INSERT INTO right_arm (timestamp, data) VALUES ($1, $2)", arm.timestamp, arm.matrices)
                .execute(pool)
                .await?;
        },
        _ => return Err(sqlx::Error::from(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid table name"))),
    }
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
            if let Err(e) = write_data_to_db(&msg.payload_str(), &msg.topic()).await {
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
