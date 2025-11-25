//! Fall Guys Log Daemon
//!
//! A daemon that monitors Fall Guys log files in real-time and parses
//! game events for further processing.
//!
//! # Overview
//!
//! This daemon:
//! - Watches the Fall Guys log directory for changes
//! - Parses log lines using nom-based parser combinators
//! - Extracts game events like matchmaking, round info, player actions
//! - Outputs parsed events for consumption by other applications
//!
//! # Configuration
//!
//! The daemon is configured via environment variables:
//! - `LOG_DIR`: Directory containing Fall Guys log files
//! - `LOG_FILE`: Name of the log file to monitor (usually `Player.log`)
//!
//! # Example
//!
//! ```bash
//! LOG_DIR=/path/to/logs LOG_FILE=Player.log cargo run
//! ```

use std::env;

use temporal_rs::options::ToStringRoundingOptions;
use tracing::{Level, debug, error, info};

pub mod error;
pub mod extra_data;
pub mod models;
pub mod parser;

/// Default log file name if not specified.
const DEFAULT_LOG_FILE: &str = "Player.log";

#[tokio::main]
async fn main() {
    // Load environment from .env file if present
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Note: .env file not loaded: {}", e);
    }
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(true)
        .with_max_level(Level::DEBUG)
        .init();

    // Get configuration from environment
    let log_dir = match env::var("LOG_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            error!("LOG_DIR environment variable not set");
            error!("Usage: LOG_DIR=/path/to/logs cargo run");
            std::process::exit(1);
        }
    };
    
    let log_file = env::var("LOG_FILE").unwrap_or_else(|_| {
        info!("LOG_FILE not set, using default: {}", DEFAULT_LOG_FILE);
        DEFAULT_LOG_FILE.to_owned()
    });
    
    let file_path = format!("{}/{}", log_dir, log_file);
    
    info!("Starting Fall Guys Log Daemon");
    info!("Watching: {}", file_path);

    // Set up the parsing pipeline
    let watch_rx = parser::task_watch::watch_dir(&log_dir, &log_file).await;
    let reader_rx = parser::task_watch::read_log_file(watch_rx, &file_path).await;
    let mut parser_rx = parser::task_parser::parse_from_str_rx(reader_rx).await;
    
    // Process parsed messages
    while let Some((message, instant)) = parser_rx.recv().await {
        let timestamp = instant
            .and_then(|i| i.to_ixdtf_string(None, ToStringRoundingOptions::default()).ok());
        debug!("{:?}: {:?}", timestamp, message);
    }
    
    info!("Log daemon shutting down");
}
