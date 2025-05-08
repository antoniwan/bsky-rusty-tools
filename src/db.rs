use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite::{Connection, params, OptionalExtension};
use std::fs;
use std::path::PathBuf;
use crate::auth::get_session;
use crate::api::BLUESKY_API_URL;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::error::Result as AppResult;
use log::info;

#[derive(Debug, Serialize, Deserialize)]
pub struct Follower {
    pub did: String,
    pub handle: String,
    pub indexed_at: DateTime<Utc>,
}

static DB_CONNECTION: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

fn get_db_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "rusty-tools", "bsky")
        .context("Failed to get project directories")?;
    
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    
    Ok(data_dir.join("followers.db"))
}

fn get_connection() -> Result<Connection> {
    let mut conn_guard = DB_CONNECTION.lock().map_err(|_| anyhow::anyhow!("Failed to lock database connection"))?;
    
    if conn_guard.is_none() {
        let db_path = get_db_path()?;
        let conn = Connection::open(db_path)?;
        init_db(&conn)?;
        *conn_guard = Some(conn);
    }
    
    // Create a new connection since rusqlite::Connection doesn't implement Clone
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)?;
    init_db(&conn)?;
    Ok(conn)
}

fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS followers (
            did TEXT PRIMARY KEY,
            handle TEXT NOT NULL,
            indexed_at DATETIME NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS follower_diffs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            did TEXT NOT NULL,
            handle TEXT NOT NULL,
            action TEXT NOT NULL,
            timestamp DATETIME NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

pub fn save_handle(handle: &str) -> AppResult<()> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO handles (handle) VALUES (?1)",
        params![handle],
    )?;
    Ok(())
}

pub fn get_saved_handle() -> AppResult<Option<String>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare("SELECT handle FROM handles ORDER BY id DESC LIMIT 1")?;
    let handle = stmt.query_row([], |row| row.get(0))
        .optional()?;
    Ok(handle)
}

// Convert DateTime to ISO8601 string for SQLite storage
fn datetime_to_sqlite(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

pub async fn save_followers() -> AppResult<()> {
    let session = get_session()?;

    println!("Fetching followers for @{}...", session.handle);

    let client = Client::new();
    let response = client
        .get(format!("{}/com.atproto.repo.getFollowers", BLUESKY_API_URL))
        .header("Authorization", format!("Bearer {}", session.access_jwt))
        .query(&[("actor", &session.handle)])
        .send()
        .await?;

    let followers: Vec<Follower> = response.json().await?;
    let followers_count = followers.len();
    
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;

    // Clear existing followers
    tx.execute("DELETE FROM followers", [])?;

    // Insert new followers
    for follower in followers {
        tx.execute(
            "INSERT INTO followers (did, handle, indexed_at) VALUES (?1, ?2, ?3)",
            params![
                follower.did,
                follower.handle,
                datetime_to_sqlite(&follower.indexed_at)
            ],
        )?;
    }

    tx.commit()?;
    println!("Saved {} followers to database", followers_count);
    Ok(())
}

pub async fn compare_followers() -> AppResult<()> {
    let session = get_session()?;

    println!("Fetching current followers for @{}...", session.handle);

    let client = Client::new();
    let response = client
        .get(format!("{}/com.atproto.repo.getFollowers", BLUESKY_API_URL))
        .header("Authorization", format!("Bearer {}", session.access_jwt))
        .query(&[("actor", &session.handle)])
        .send()
        .await?;

    let current_followers: Vec<Follower> = response.json().await?;
    
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;

    // Get previous followers
    let mut stmt = tx.prepare("SELECT did, handle FROM followers")?;
    let previous_followers: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;

    let current_dids: std::collections::HashSet<_> = current_followers
        .iter()
        .map(|f| &f.did)
        .collect();

    let previous_dids: std::collections::HashSet<_> = previous_followers
        .iter()
        .map(|(did, _)| did)
        .collect();

    // Find new followers
    for follower in &current_followers {
        if !previous_dids.contains(&follower.did) {
            println!("🆕 New follower: @{}", follower.handle);
            tx.execute(
                "INSERT INTO follower_diffs (did, handle, action, timestamp) VALUES (?1, ?2, ?3, ?4)",
                params![
                    follower.did,
                    follower.handle,
                    "follow",
                    datetime_to_sqlite(&Utc::now())
                ],
            )?;
        }
    }

    // Find unfollowers
    for (did, handle) in &previous_followers {
        if !current_dids.contains(did) {
            println!("❌ Unfollower: @{}", handle);
            tx.execute(
                "INSERT INTO follower_diffs (did, handle, action, timestamp) VALUES (?1, ?2, ?3, ?4)",
                params![
                    did,
                    handle,
                    "unfollow",
                    datetime_to_sqlite(&Utc::now())
                ],
            )?;
        }
    }

    drop(stmt); // Drop the statement before committing
    tx.commit()?;
    Ok(())
} 