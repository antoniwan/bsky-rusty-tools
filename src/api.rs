use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::auth::get_session;

pub const BLUESKY_API_URL: &str = "https://bsky.social/xrpc";

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    handle: String,
    did: String,
    display_name: Option<String>,
    description: Option<String>,
    followers_count: i32,
    follows_count: i32,
}

pub async fn print_my_profile() -> Result<()> {
    let session = get_session()?
        .context("Not logged in. Please run 'login' first")?;

    let client = Client::new();
    let response = client
        .get(format!("{}/com.atproto.repo.getProfile", BLUESKY_API_URL))
        .header("Authorization", format!("Bearer {}", session.access_jwt))
        .query(&[("actor", session.handle)])
        .send()
        .await?;

    let profile: Profile = response.json().await?;
    
    println!("Profile for @{}", profile.handle);
    println!("DID: {}", profile.did);
    if let Some(name) = profile.display_name {
        println!("Display Name: {}", name);
    }
    if let Some(desc) = profile.description {
        println!("Description: {}", desc);
    }
    println!("Followers: {}", profile.followers_count);
    println!("Following: {}", profile.follows_count);

    Ok(())
}

pub async fn lookup_profile(handle: &str) -> Result<()> {
    let session = get_session()?
        .context("Not logged in. Please run 'login' first")?;

    let client = Client::new();
    let response = client
        .get(format!("{}/com.atproto.repo.getProfile", BLUESKY_API_URL))
        .header("Authorization", format!("Bearer {}", session.access_jwt))
        .query(&[("actor", handle)])
        .send()
        .await?;

    let profile: Profile = response.json().await?;
    
    println!("Profile for @{}", profile.handle);
    println!("DID: {}", profile.did);
    if let Some(name) = profile.display_name {
        println!("Display Name: {}", name);
    }
    if let Some(desc) = profile.description {
        println!("Description: {}", desc);
    }
    println!("Followers: {}", profile.followers_count);
    println!("Following: {}", profile.follows_count);

    Ok(())
}

pub async fn mirror_follows(handle: &str) -> Result<()> {
    let session = get_session()?
        .context("Not logged in. Please run 'login' first")?;

    println!("Fetching follows for @{}...", handle);
    
    // TODO: Implement actual follow mirroring
    // This will require:
    // 1. Fetching the user's follows
    // 2. Checking which ones we don't follow
    // 3. Following them one by one with rate limiting
    
    println!("Mirror follows functionality coming soon!");
    Ok(())
} 