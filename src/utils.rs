use chrono::{DateTime, Utc};
use std::time::Duration;
use anyhow::Result;

// These functions are kept for future use but marked as #[allow(dead_code)]
#[allow(dead_code)]
pub fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

#[allow(dead_code)]
pub fn parse_timestamp(s: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc))
}

#[allow(dead_code)]
pub async fn rate_limit(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[allow(dead_code)]
pub fn handle_api_error(status: u16, body: &str) -> anyhow::Error {
    anyhow::anyhow!("API error (status {}): {}", status, body)
} 