use crate::error::{AppError, Result as AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::auth::get_session;
use log::{info, error};

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

#[derive(Debug, Deserialize)]
pub struct Follower {
    pub did: String,
    pub handle: String,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
}

pub async fn print_my_profile() -> AppResult<()> {
    let session = get_session()?;

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

pub async fn lookup_profile(handle: &str) -> AppResult<()> {
    let session = get_session()?;

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

pub async fn mirror_follows(handle: &str) -> AppResult<()> {
    let _session = get_session()?;

    println!("Fetching follows for @{}...", handle);
    
    // TODO: Implement actual follow mirroring
    // This will require:
    // 1. Fetching the user's follows
    // 2. Checking which ones we don't follow
    // 3. Following them one by one with rate limiting
    
    println!("Mirror follows functionality coming soon!");
    Ok(())
}

pub async fn get_followers() -> AppResult<Vec<Follower>> {
    let session = get_session()?;
    let client = Client::new();
    
    let response = client
        .get(format!("{}/app.bsky.graph.getFollowers", BLUESKY_API_URL))
        .header("Authorization", format!("Bearer {}", session.access_jwt))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        error!("Failed to get followers: {}", error_text);
        return Err(AppError::Api(error_text));
    }

    let followers: Vec<Follower> = response.json().await?;
    Ok(followers)
}

pub async fn get_following() -> AppResult<Vec<Follower>> {
    let session = get_session()?;
    let client = Client::new();
    
    let response = client
        .get(format!("{}/app.bsky.graph.getFollows", BLUESKY_API_URL))
        .header("Authorization", format!("Bearer {}", session.access_jwt))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        error!("Failed to get following: {}", error_text);
        return Err(AppError::Api(error_text));
    }

    let following: Vec<Follower> = response.json().await?;
    Ok(following)
}

pub async fn follow(did: &str) -> AppResult<()> {
    let session = get_session()?;
    let client = Client::new();
    
    let response = client
        .post(format!("{}/app.bsky.graph.follow", BLUESKY_API_URL))
        .header("Authorization", format!("Bearer {}", session.access_jwt))
        .json(&serde_json::json!({
            "subject": did,
            "createdAt": chrono::Utc::now().to_rfc3339()
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        error!("Failed to follow user: {}", error_text);
        return Err(AppError::Api(error_text));
    }

    Ok(())
}

pub async fn unfollow(did: &str) -> AppResult<()> {
    let session = get_session()?;
    let client = Client::new();
    
    let response = client
        .post(format!("{}/app.bsky.graph.unfollow", BLUESKY_API_URL))
        .header("Authorization", format!("Bearer {}", session.access_jwt))
        .json(&serde_json::json!({
            "subject": did,
            "createdAt": chrono::Utc::now().to_rfc3339()
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        error!("Failed to unfollow user: {}", error_text);
        return Err(AppError::Api(error_text));
    }

    Ok(())
} 