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
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Represents a BlueSky user session with authentication tokens and user information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub access_jwt: String,
    pub refresh_jwt: String,
    pub handle: String,
    pub did: String,
    pub email: String,
}

/// Session manager to handle authentication state
#[derive(Debug)]
pub struct SessionManager {
    session: Arc<Mutex<Option<Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_session(&self) -> Result<Session> {
        let mut session_guard = self.session.lock()
            .map_err(|_| AppError::Auth("Failed to lock session".to_string()))?;
        
        if session_guard.is_none() {
            let session_path = get_session_path()?;
            let json = fs::read_to_string(session_path)?;
            let session: Session = serde_json::from_str(&json)?;
            *session_guard = Some(session.clone());
            Ok(session)
        } else {
            Ok(session_guard.as_ref().unwrap().clone())
        }
    }

    pub fn save_session(&self, session: &Session) -> Result<()> {
        let session_path = get_session_path()?;
        let json = serde_json::to_string_pretty(session)?;
        fs::write(session_path, json)?;
        *self.session.lock()
            .map_err(|_| AppError::Auth("Failed to lock session".to_string()))? = Some(session.clone());
        info!("Session saved successfully");
        Ok(())
    }

    pub fn clear_session(&self) -> Result<()> {
        let session_path = get_session_path()?;
        if session_path.exists() {
            fs::remove_file(session_path)?;
            *self.session.lock()
                .map_err(|_| AppError::Auth("Failed to lock session".to_string()))? = None;
            info!("Successfully logged out");
        } else {
            warn!("No active session found");
        }
        Ok(())
    }
}

// Global session manager instance
static SESSION_MANAGER: Lazy<SessionManager> = Lazy::new(|| SessionManager::new());

#[derive(Debug, Serialize)]
struct LoginRequest {
    identifier: String,
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    app_password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_code: Option<String>,
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
        .ok_or_else(|| AppError::Config("Failed to get project directories".to_string()))?;
    
    let data_dir = proj_dirs.data_dir();
    fs::create_dir_all(data_dir)
        .map_err(|e| AppError::Io(e))?;
    
    Ok(data_dir.join("session.json"))
}

pub async fn login(handle: &str) -> Result<Session> {
    info!("Attempting login for handle: {}", handle);
    let password = prompt_password("Enter your app password: ")?;
    let password_clone = password.clone();

    let client = Client::new();
    let mut response = client
        .post(format!("{}/com.atproto.server.createSession", BLUESKY_API_URL))
        .json(&LoginRequest {
            identifier: handle.to_string(),
            password: password_clone.clone(),
            app_password: Some(password_clone),
            verification_code: None,
        })
        .send()
        .await?;

    // Handle email verification if required
    if !response.status().is_success() {
        let error_text = response.text().await?;
        if error_text.contains("AuthFactorTokenRequired") {
            let verification_code = prompt_password("Enter the verification code sent to your email: ")?;
            
            response = client
                .post(format!("{}/com.atproto.server.createSession", BLUESKY_API_URL))
                .json(&LoginRequest {
                    identifier: handle.to_string(),
                    password: password.clone(),
                    app_password: Some(password),
                    verification_code: Some(verification_code),
                })
                .send()
                .await?;
        } else {
            error!("Login failed: {}", error_text);
            return Err(AppError::Auth(error_text));
        }
    }

    if !response.status().is_success() {
        let error_text = response.text().await?;
        error!("Login failed: {}", error_text);
        return Err(AppError::Auth(error_text));
    }

    let login_response: LoginResponse = response.json().await?;
    
    let session = Session {
        access_jwt: login_response.access_jwt,
        refresh_jwt: login_response.refresh_jwt,
        handle: login_response.handle,
        did: login_response.did,
        email: String::new(), // TODO: Get email from API if available
    };
    
    SESSION_MANAGER.save_session(&session)?;
    info!("Successfully logged in as {}", session.handle);
    Ok(session)
}

pub fn logout() -> Result<()> {
    SESSION_MANAGER.clear_session()
}

pub fn get_session() -> Result<Session> {
    SESSION_MANAGER.get_session()
}

pub fn get_handle() -> Result<String> {
    // First try to get the handle from the database
    match get_saved_handle() {
        Ok(Some(handle)) => {
            info!("Using saved handle: {}", handle);
            Ok(handle)
        }
        Ok(None) => {
            // If not found in database, prompt the user
            print!("Enter your BlueSky handle (e.g. user.bsky.social): ");
            std::io::stdout().flush()
                .map_err(AppError::Io)?;
            
            let mut handle = String::new();
            std::io::stdin().read_line(&mut handle)
                .map_err(AppError::Io)?;
            let handle = handle.trim().to_string();

            // Save the handle to the database for future use
            save_handle(&handle)?;
            info!("Saved new handle: {}", handle);

            Ok(handle)
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::env;

    fn setup_test_env() -> tempfile::TempDir {
        let temp_dir = tempdir().unwrap();
        env::set_var("HOME", temp_dir.path());
        temp_dir
    }

    #[test]
    fn test_session_manager() {
        let _temp_dir = setup_test_env();
        let manager = SessionManager::new();
        
        let test_session = Session {
            access_jwt: "test_access".to_string(),
            refresh_jwt: "test_refresh".to_string(),
            handle: "test.bsky.social".to_string(),
            did: "test_did".to_string(),
            email: "test@example.com".to_string(),
        };

        // Test saving and retrieving session
        manager.save_session(&test_session).unwrap();
        let retrieved = manager.get_session().unwrap();
        assert_eq!(retrieved.handle, test_session.handle);
        assert_eq!(retrieved.did, test_session.did);

        // Test clearing session
        manager.clear_session().unwrap();
        assert!(manager.get_session().is_err());
    }

    #[test]
    fn test_get_session_path() {
        let _temp_dir = setup_test_env();
        let path = get_session_path().unwrap();
        assert!(path.to_string_lossy().contains("bsky"));
        assert!(path.to_string_lossy().contains("session.json"));
    }
} 