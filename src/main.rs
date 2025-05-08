mod api;
mod auth;
mod db;
mod error;
mod utils;

use error::{AppError, Result};
use clap::{Parser, Subcommand};
use crate::auth::{login, get_handle};
use crate::db::save_followers;
use log::{info, error};

/// BlueSky CLI toolset for automation
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Login to BlueSky with your credentials
    Login,
    /// Save your followers to a local database
    SaveFollowers,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting bsky-rusty-tools");

    let cli = Cli::parse();

    match cli.command {
        Commands::Login => {
            let handle = get_handle()?;
            match login(&handle).await {
                Ok(session) => {
                    info!("Successfully logged in as {}", session.handle);
                }
                Err(e) => {
                    error!("Login failed: {}", e);
                    return Err(AppError::Auth(format!("Login failed: {}", e)));
                }
            }
        }
        Commands::SaveFollowers => {
            match save_followers().await {
                Ok(_) => {
                    info!("Successfully saved followers");
                }
                Err(e) => {
                    error!("Failed to save followers: {}", e);
                    return Err(AppError::Api(format!("Failed to save followers: {}", e)));
                }
            }
        }
    }

    Ok(())
}
