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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Login to BlueSky
    Login,
    /// Save followers to database
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
                    return Err(e);
                }
            }
        }
        Commands::SaveFollowers => {
            if let Err(e) = save_followers().await {
                error!("Failed to save followers: {}", e);
                return Err(e);
            }
            info!("Successfully saved followers");
        }
    }

    Ok(())
}
