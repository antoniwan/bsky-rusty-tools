use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite::{Connection, params};
use std::fs;
use std::path::PathBuf;
use crate::auth::get_session;
use crate::api::BLUESKY_API_URL;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Follower {
    did: String,
    handle: String,
    indexed_at: DateTime<Utc>,
}

fn get_db_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "rusty-tools", "bsky")
        .context("Failed to get project directories")?;
    
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    
    Ok(data_dir.join("followers.db"))
}

fn init_db() -> Result<Connection> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)?;

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

    Ok(conn)
}

pub async fn save_followers() -> Result<()> {
    let session = get_session()?
        .context("Not logged in. Please run 'login' first")?;

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
    
    let mut conn = init_db()?;
    let tx = conn.transaction()?;

    // Clear existing followers
    tx.execute("DELETE FROM followers", [])?;

    // Insert new followers
    for follower in followers {
        tx.execute(
            "INSERT INTO followers (did, handle, indexed_at) VALUES (?1, ?2, ?3)",
            params![follower.did, follower.handle, follower.indexed_at],
        )?;
    }

    tx.commit()?;
    println!("Saved {} followers to database", followers_count);
    Ok(())
}

pub async fn compare_followers() -> Result<()> {
    let session = get_session()?
        .context("Not logged in. Please run 'login' first")?;

    println!("Fetching current followers for @{}...", session.handle);

    let client = Client::new();
    let response = client
        .get(format!("{}/com.atproto.repo.getFollowers", BLUESKY_API_URL))
        .header("Authorization", format!("Bearer {}", session.access_jwt))
        .query(&[("actor", &session.handle)])
        .send()
        .await?;

    let current_followers: Vec<Follower> = response.json().await?;
    
    let mut conn = init_db()?;
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
                params![follower.did, follower.handle, "follow", Utc::now()],
            )?;
        }
    }

    // Find unfollowers
    for (did, handle) in &previous_followers {
        if !current_dids.contains(did) {
            println!("❌ Unfollower: @{}", handle);
            tx.execute(
                "INSERT INTO follower_diffs (did, handle, action, timestamp) VALUES (?1, ?2, ?3, ?4)",
                params![did, handle, "unfollow", Utc::now()],
            )?;
        }
    }

    drop(stmt); // Drop the statement before committing
    tx.commit()?;
    Ok(())
} 