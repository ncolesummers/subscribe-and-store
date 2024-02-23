use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use once_cell::sync::Lazy;
use std::env;

static POOL: Lazy<Pool<Postgres>> = Lazy::new(|| {
    // Database connection setup...
});

pub async fn write_data_to_db(data: &str, table: &str) -> Result<(), sqlx::Error> {
    // Your data insertion logic...
}
