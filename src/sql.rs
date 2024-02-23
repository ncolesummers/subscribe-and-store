use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use once_cell::sync::Lazy;
use anyhow::{bail, Context, Result};

static POOL: Lazy<Pool<Postgres>> = Lazy::new(|| {
    let database_url = &dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5) // Example: set maximum connections to 5
        .connect_lazy(database_url)
        .expect("Failed to create pool")
});

pub async fn write_data_to_db(data: &str, table: &str) -> Result<(), sqlx::Error> {
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
) -> Result<()> {
    // Parse the epoch timestamp from the ArmMessage, assuming it's in milliseconds
    let epoch_milliseconds = arm.timestamp;

    // Convert milliseconds to seconds and nanoseconds
    let seconds = epoch_milliseconds / 1000;
    let nanoseconds = (epoch_milliseconds % 1000) * 1_000_000; // Convert remainder to nanoseconds

    // Create OffsetDateTime from seconds and nanoseconds
    let parsed_timestamp = OffsetDateTime::from_unix_timestamp(seconds)
        .map(|dt| dt + Duration::nanoseconds(nanoseconds))
        .map_err(|_| anyhow::anyhow!("Timestamp out of range"))?;

    // Serialize the matrices to a JSON string
    let matrices_json = serde_json::to_string(&arm.matrices)
    .context("Failed to serialize matrices to JSON")?;

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
