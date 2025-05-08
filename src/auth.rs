use anyhow::{Result, Context};
use directories::ProjectDirs;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::api::BLUESKY_API_URL;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub access_jwt: String,
    pub refresh_jwt: String,
    pub handle: String,
    pub did: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    identifier: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    access_jwt: String,
    refresh_jwt: String,
    handle: String,
    did: String,
}

pub fn get_session_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "rusty-tools", "bsky")
        .context("Failed to get project directories")?;
    
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    
    Ok(data_dir.join("session.json"))
}

pub async fn login() -> Result<()> {
    println!("Please enter your BlueSky handle:");
    let mut handle = String::new();
    std::io::stdin().read_line(&mut handle)?;
    let handle = handle.trim();

    println!("Please enter your app password:");
    let mut password = String::new();
    std::io::stdin().read_line(&mut password)?;
    let password = password.trim();

    let client = Client::new();
    let response = client
        .post(format!("{}/com.atproto.server.createSession", BLUESKY_API_URL))
        .json(&LoginRequest {
            identifier: handle.to_string(),
            password: password.to_string(),
        })
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Login failed: {}", error_text);
    }

    let login_response: LoginResponse = response.json().await?;
    let session = Session {
        access_jwt: login_response.access_jwt,
        refresh_jwt: login_response.refresh_jwt,
        handle: login_response.handle,
        did: login_response.did,
    };

    let session_path = get_session_path()?;
    fs::write(
        session_path,
        serde_json::to_string_pretty(&session)?
    )?;

    println!("Successfully logged in as {}", session.handle);
    Ok(())
}

pub fn logout() -> Result<()> {
    let session_path = get_session_path()?;
    if session_path.exists() {
        fs::remove_file(session_path)?;
        println!("Successfully logged out");
    } else {
        println!("No active session found");
    }
    Ok(())
}

pub fn get_session() -> Result<Option<Session>> {
    let session_path = get_session_path()?;
    if !session_path.exists() {
        return Ok(None);
    }

    let session_str = fs::read_to_string(session_path)?;
    let session: Session = serde_json::from_str(&session_str)?;
    Ok(Some(session))
} 