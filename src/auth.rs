use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use crate::api::BLUESKY_API_URL;
use crate::db::{save_handle, get_saved_handle};
use rpassword::prompt_password;
use log::{info, warn, error};
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub access_jwt: String,
    pub refresh_jwt: String,
    pub handle: String,
    pub did: String,
    pub email: String,
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

static CURRENT_SESSION: Lazy<Mutex<Option<Session>>> = Lazy::new(|| Mutex::new(None));

pub fn get_session_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "rusty-tools", "bsky")
        .ok_or_else(|| AppError::Config("Failed to get project directories".to_string()))?;
    
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir)
        .map_err(|e| AppError::Io(e))?;
    
    Ok(data_dir.join("session.json"))
}

pub async fn login(handle: &str) -> Result<Session> {
    info!("Attempting login for handle: {}", handle);
    let password = prompt_password("Enter your password: ")
        .map_err(|e| AppError::Auth(format!("Failed to read password: {}", e)))?;

    let client = Client::new();
    let response = client
        .post(format!("{}/com.atproto.server.createSession", BLUESKY_API_URL))
        .json(&LoginRequest {
            identifier: handle.to_string(),
            password,
        })
        .send()
        .await
        .map_err(|e| AppError::Api(format!("Failed to send login request: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await
            .map_err(|e| AppError::Api(format!("Failed to read error response: {}", e)))?;
        error!("Login failed: {}", error_text);
        return Err(AppError::Auth(error_text));
    }

    let login_response: LoginResponse = response.json().await
        .map_err(|e| AppError::Api(format!("Failed to parse login response: {}", e)))?;
    
    let session = Session {
        access_jwt: login_response.access_jwt,
        refresh_jwt: login_response.refresh_jwt,
        handle: login_response.handle,
        did: login_response.did,
        email: String::new(), // TODO: Get email from API if available
    };
    
    save_session(&session)?;
    info!("Successfully logged in as {}", session.handle);
    Ok(session)
}

pub fn logout() -> Result<()> {
    let session_path = get_session_path()?;
    if session_path.exists() {
        fs::remove_file(session_path)
            .map_err(|e| AppError::Io(e))?;
        *CURRENT_SESSION.lock()
            .map_err(|_| AppError::Auth("Failed to lock session".to_string()))? = None;
        info!("Successfully logged out");
    } else {
        warn!("No active session found");
    }
    Ok(())
}

pub fn get_session() -> Result<Session> {
    let mut session_guard = CURRENT_SESSION.lock()
        .map_err(|_| AppError::Auth("Failed to lock session".to_string()))?;
    
    if session_guard.is_none() {
        let session_path = get_session_path()?;
        let json = fs::read_to_string(session_path)
            .map_err(|e| AppError::Io(e))?;
        let session: Session = serde_json::from_str(&json)
            .map_err(|e| AppError::Serialization(e))?;
        *session_guard = Some(session.clone());
        Ok(session)
    } else {
        Ok(session_guard.as_ref().unwrap().clone())
    }
}

pub fn get_handle() -> Result<String> {
    // First try to get the handle from the database
    if let Some(handle) = get_saved_handle()? {
        info!("Using saved handle: {}", handle);
        return Ok(handle);
    }

    // If not found in database, prompt the user
    print!("Enter your BlueSky handle (e.g. user.bsky.social): ");
    std::io::stdout().flush()
        .map_err(|e| AppError::Io(e))?;
    
    let mut handle = String::new();
    std::io::stdin().read_line(&mut handle)
        .map_err(|e| AppError::Io(e))?;
    let handle = handle.trim().to_string();

    // Save the handle to the database for future use
    save_handle(&handle)?;
    info!("Saved new handle: {}", handle);

    Ok(handle)
}

pub fn save_session(session: &Session) -> Result<()> {
    let session_path = get_session_path()?;
    let json = serde_json::to_string_pretty(session)
        .map_err(|e| AppError::Serialization(e))?;
    fs::write(session_path, json)
        .map_err(|e| AppError::Io(e))?;
    *CURRENT_SESSION.lock()
        .map_err(|_| AppError::Auth("Failed to lock session".to_string()))? = Some(session.clone());
    info!("Session saved successfully");
    Ok(())
} 