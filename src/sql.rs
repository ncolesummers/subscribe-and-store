use crate::model::ArmMessage;
use anyhow::{bail, Context, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use time::{Duration, OffsetDateTime};

pub fn init_pg_pool() -> Result<Pool<Postgres>> {
    let database_url = &dotenv::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(5) // Example: set maximum connections to 5
        .connect_lazy(database_url)
        .context("Failed to create pool")?;
    Ok(pool)
}

pub async fn write_data_to_db(data: &str, table: &str, pool: &Pool<Postgres>) -> Result<()> {
    // Data is a JSON string, so you can parse it into a struct using serde_json
    let arm: ArmMessage = serde_json::from_str(data)
        .with_context(|| format!("Failed to parse data as ArmMessage JSON: {}", data))?;

    insert_data(table, arm, pool).await?;

    Ok(())
}

async fn insert_data(table: &str, arm: ArmMessage, pool: &Pool<Postgres>) -> Result<()> {
    // Parse the epoch timestamp from the ArmMessage, assuming it's in milliseconds
    let epoch_milliseconds = arm.timestamp;

    // Convert milliseconds to seconds and nanoseconds
    let seconds = epoch_milliseconds / 1000;
    let nanoseconds = (epoch_milliseconds % 1000) * 1_000_000; // Convert remainder to nanoseconds
    println!("Seconds: {}, Nanoseconds: {}", seconds, nanoseconds);

    // Create OffsetDateTime from seconds and nanoseconds
    let parsed_timestamp = OffsetDateTime::from_unix_timestamp(seconds)
        .map(|dt| dt + Duration::nanoseconds(nanoseconds))
        .map_err(|_| anyhow::anyhow!("Timestamp out of range"))?;

    // Serialize the matrices to a JSON string
    let matrices_json =
        serde_json::to_string(&arm.matrices).context("Failed to serialize matrices to JSON")?;

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
                .await
                .with_context(|| format!("Failed to insert data into table {}", table))?;
        }
        _ => {
            // Use `bail!` to return an error immediately with a formatted message
            bail!("Invalid table name: {}", table);
        }
    }
    Ok(())
}
