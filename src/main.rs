mod auth;
mod db;
mod api;
mod utils;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "Rusty Tools", version, author, about = "BlueSky CLI Toolkit in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with Bluesky
    Login,
    /// Logout and clear credentials
    Logout,
    /// Show your profile info
    Me,
    /// Save your current followers into SQLite
    SaveFollowers,
    /// Compare last followers snapshot vs current
    CompareFollowers,
    /// Get info for any handle
    Lookup { handle: String },
    /// Follow everyone a user is following
    MirrorFollows { handle: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Login => auth::login().await?,
        Commands::Logout => auth::logout()?,
        Commands::Me => api::print_my_profile().await?,
        Commands::SaveFollowers => db::save_followers().await?,
        Commands::CompareFollowers => db::compare_followers().await?,
        Commands::Lookup { handle } => api::lookup_profile(&handle).await?,
        Commands::MirrorFollows { handle } => api::mirror_follows(&handle).await?,
    }

    Ok(())
}
