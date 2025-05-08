use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub fn parse_timestamp(s: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %z")?.with_timezone(&Utc))
}

pub async fn rate_limit(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

pub fn handle_api_error(status: u16, body: &str) -> anyhow::Error {
    match serde_json::from_str::<ErrorResponse>(body) {
        Ok(err) => anyhow::anyhow!("API Error ({}): {}", status, err.message),
        Err(_) => anyhow::anyhow!("API Error ({}): {}", status, body),
    }
} 