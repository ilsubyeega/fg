//! File system watching for Fall Guys log files.
//!
//! This module provides async file watching capabilities to monitor
//! Fall Guys log files for changes in real-time.
//!
//! # Architecture
//!
//! The watcher operates in two stages:
//!
//! 1. **Directory Watch** ([`watch_dir`]): Monitors the log directory for
//!    file creation, modification, and close events.
//!
//! 2. **File Reader** ([`read_log_file`]): Reads new content from the log
//!    file when changes are detected, streaming individual lines.
//!
//! # Platform Notes
//!
//! Fall Guys on Linux/Steam/Proton generates the following event sequence:
//!
//! ```text
//! Access(Open(Any)) → Access(Close(Write)) → Access(Open(Any)) →
//! Modify(Name(From)) → Modify(Name(To)) → Modify(Name(Both)) →
//! Access(Open(Any)) → Create(File) → Modify(Data(Any)) →
//! Access(Close(Write))
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use fg_daemon::parser::task_watch::{watch_dir, read_log_file};
//!
//! let watch_rx = watch_dir("/path/to/logs", "Player.log").await;
//! let line_rx = read_log_file(watch_rx, "/path/to/logs/Player.log").await;
//!
//! while let Some(line) = line_rx.recv().await {
//!     println!("New log line: {}", line);
//! }
//! ```

use notify::{
    Config, EventKind, RecommendedWatcher, Watcher,
    event::{AccessKind, CreateKind, ModifyKind},
};
use std::io::ErrorKind;
use std::path::Path;
use tokio::{
    fs::OpenOptions,
    io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom},
    sync::mpsc::{self, Receiver},
};
use tracing::{debug, error, warn};

// =============================================================================
// Types
// =============================================================================

/// Messages sent from the file watcher to the reader.
///
/// These messages inform the log reader about the state of the log file,
/// allowing it to respond appropriately to file changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchMessage {
    /// A new log file was created.
    ///
    /// This typically happens when Fall Guys starts a new session.
    /// The reader should reset its position to the beginning.
    FileCreated,

    /// The log file content was modified.
    ///
    /// Contains the new total length of the file, allowing the reader
    /// to calculate how much new content to read.
    ContentModified {
        /// The new total length of the file in bytes.
        length: u64,
    },

    /// The log file was closed by the writing process.
    ///
    /// This may indicate the game has finished writing or is shutting down.
    Closed,
}

// =============================================================================
// Async Watcher
// =============================================================================

/// Creates an async-compatible file system watcher.
///
/// This wraps the synchronous `notify` watcher to work with tokio's async runtime
/// by using a channel to bridge the sync callback to async code.
///
/// # Returns
///
/// A tuple of (watcher, receiver) where:
/// - `watcher` should be kept alive as long as you want to receive events
/// - `receiver` yields file system events
///
/// # Errors
///
/// Returns an error if the underlying watcher fails to initialize.
pub fn async_watcher()
-> notify::Result<(RecommendedWatcher, Receiver<notify::Result<notify::Event>>)> {
    let (tx, rx) = mpsc::channel(1024);
    
    let watcher = RecommendedWatcher::new(
        move |res| {
            // Use blocking_send since notify callbacks are synchronous
            if let Err(e) = tx.blocking_send(res) {
                error!("Failed to send watch event: {}", e);
            }
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

// =============================================================================
// Directory Watch
// =============================================================================

/// Watches a directory for changes to a specific log file.
///
/// This function sets up a file system watcher on the specified directory
/// and filters events to only those affecting the target log file.
///
/// # Arguments
///
/// * `dir_path` - The directory to watch (e.g., `~/.local/share/Steam/...`)
/// * `file_name` - The name of the log file to monitor (e.g., `Player.log`)
///
/// # Returns
///
/// A channel that yields [`WatchMessage`] events when the log file changes.
///
/// # Example
///
/// ```rust,ignore
/// let watch_rx = watch_dir("/path/to/logs", "Player.log").await;
///
/// while let Some(msg) = watch_rx.recv().await {
///     match msg {
///         WatchMessage::FileCreated => println!("New log file"),
///         WatchMessage::ContentModified { length } => println!("New length: {}", length),
///         WatchMessage::Closed => println!("File closed"),
///     }
/// }
/// ```
pub async fn watch_dir(dir_path: &str, file_name: &str) -> Receiver<WatchMessage> {
    let (tx, rx) = mpsc::channel(1024);
    
    let (mut watcher, mut watch_rx) = match async_watcher() {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to create file watcher: {}", e);
            return rx;
        }
    };

    if let Err(e) = watcher.watch(Path::new(dir_path), notify::RecursiveMode::NonRecursive) {
        error!("Failed to watch directory {}: {}", dir_path, e);
        return rx;
    }

    let file_path = format!("{}/{}", dir_path, file_name);
    let dir_path = dir_path.to_owned();
    let file_name = file_name.to_owned();
    
    tokio::spawn(async move {
        // Keep watcher alive for the duration of the task
        let mut _watcher = watcher;
        
        while let Some(event_result) = watch_rx.recv().await {
            let event = match event_result {
                Ok(e) => e,
                Err(e) => {
                    warn!("Watch error: {:?}", e);
                    continue;
                }
            };

            // Handle rescan requests
            if event.need_rescan() {
                if let Err(e) = _watcher.watch(
                    Path::new(&dir_path),
                    notify::RecursiveMode::NonRecursive,
                ) {
                    error!("Failed to re-watch directory: {}", e);
                }
                debug!("Rescanning directory: {}", dir_path);
            }

            // Filter for our target file
            let Some(path) = event.paths.first() else {
                continue;
            };
            
            if !path.ends_with(&file_name) {
                continue;
            }

            // Convert event to our message type
            let msg = match event.kind {
                EventKind::Access(AccessKind::Close(_)) => Some(WatchMessage::Closed),
                EventKind::Create(CreateKind::File) => Some(WatchMessage::FileCreated),
                EventKind::Modify(ModifyKind::Data(_)) => {
                    match OpenOptions::new().read(true).open(&file_path).await {
                        Ok(file) => {
                            match file.metadata().await {
                                Ok(meta) => Some(WatchMessage::ContentModified { 
                                    length: meta.len() 
                                }),
                                Err(e) => {
                                    warn!("Failed to get file metadata: {}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to open file for length check: {}", e);
                            None
                        }
                    }
                }
                _ => None,
            };

            if let Some(msg) = msg {
                if tx.send(msg).await.is_err() {
                    debug!("Watch channel closed, stopping watcher");
                    break;
                }
            }
        }
    });

    rx
}

// =============================================================================
// Log File Reader
// =============================================================================

/// Reads new content from a log file as lines.
///
/// This function maintains a read position in the log file and reads
/// new content incrementally as the file grows. It handles file rotation
/// (when a new file is created) by resetting the read position.
///
/// # Arguments
///
/// * `watch_rx` - Channel receiving [`WatchMessage`] events from [`watch_dir`]
/// * `file_path` - Full path to the log file to read
///
/// # Returns
///
/// A channel that yields individual log lines as `String`s.
///
/// # Example
///
/// ```rust,ignore
/// let watch_rx = watch_dir("/path/to/logs", "Player.log").await;
/// let line_rx = read_log_file(watch_rx, "/path/to/logs/Player.log").await;
///
/// while let Some(line) = line_rx.recv().await {
///     // Process each line
///     println!("{}", line);
/// }
/// ```
pub async fn read_log_file(
    mut watch_rx: Receiver<WatchMessage>,
    file_path: &str,
) -> Receiver<String> {
    let (tx, rx) = mpsc::channel(1024);
    let file_path = file_path.to_owned();
    
    tokio::spawn(async move {
        // Initialize buffer position from existing file
        let mut buffer_pos = match OpenOptions::new().read(true).open(&file_path).await {
            Ok(file) => file.metadata().await.map(|f| f.len()).unwrap_or(0),
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    warn!("Error reading initial log file length: {:?}", e);
                }
                0
            }
        };

        buffer_pos = 0;

        debug!("Initialized log file reader at position: {}", buffer_pos);

        while let Some(watch_msg) = watch_rx.recv().await {
            // Handle file creation (rotation) - reset position
            if watch_msg == WatchMessage::FileCreated {
                debug!("New log file created, resetting read position");
                buffer_pos = 0;
                continue;
            }

            // Handle content modification
            let WatchMessage::ContentModified { length } = watch_msg else {
                continue;
            };

            // Detect file truncation/rotation
            if length < buffer_pos {
                warn!(
                    "File appears truncated (length {} < position {}), resetting",
                    length, buffer_pos
                );
                buffer_pos = 0;
            }

            // Skip if no new content
            if length == buffer_pos {
                continue;
            }

            // Read new content
            let mut file = match OpenOptions::new().read(true).open(&file_path).await {
                Ok(f) => f,
                Err(e) => {
                    warn!("Failed to open log file: {}", e);
                    continue;
                }
            };

            if let Err(e) = file.seek(SeekFrom::Start(buffer_pos)).await {
                warn!("Failed to seek in log file: {}", e);
                continue;
            }

            let reader = BufReader::new(file);
            let content = reader.take(length - buffer_pos);
            let mut lines = content.lines();

            // Send each line to the output channel
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        if tx.send(line).await.is_err() {
                            debug!("Line channel closed, stopping reader");
                            return;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        warn!("Error reading log line: {}", e);
                        break;
                    }
                }
            }

            buffer_pos = length;
        }
    });

    rx
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_message_equality() {
        assert_eq!(WatchMessage::FileCreated, WatchMessage::FileCreated);
        assert_eq!(
            WatchMessage::ContentModified { length: 100 },
            WatchMessage::ContentModified { length: 100 }
        );
        assert_ne!(
            WatchMessage::ContentModified { length: 100 },
            WatchMessage::ContentModified { length: 200 }
        );
    }
}
