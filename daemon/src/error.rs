//! Error types for the FG Daemon application.
//!
//! This module provides a centralized error handling system using `thiserror`
//! for defining error types and `anyhow` for error propagation.
//!
//! # Error Categories
//!
//! - [`ConfigError`]: Configuration and environment-related errors
//! - [`ExtraDataError`]: Errors loading game data assets (JSON files)
//! - [`ParserError`]: Log parsing errors
//! - [`WatcherError`]: File watching and I/O errors
//!
//! # Example
//!
//! ```rust
//! use fg_daemon::error::{ParserError, Result};
//!
//! fn parse_something() -> Result<()> {
//!     Err(ParserError::BufferOverflow { lines: 100 }.into())
//! }
//! ```

use thiserror::Error;

/// A type alias for `anyhow::Result<T>` used throughout the application.
pub type Result<T> = anyhow::Result<T>;

/// Errors that can occur during configuration loading.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// A required environment variable is missing.
    #[error("Missing environment variable: {name}")]
    MissingEnvVar {
        /// The name of the missing environment variable.
        name: &'static str,
    },

    /// An environment variable has an invalid value.
    #[error("Invalid value for {name}: {message}")]
    InvalidValue {
        /// The name of the environment variable.
        name: &'static str,
        /// Description of what's wrong with the value.
        message: String,
    },
}

/// Errors that can occur when loading extra data assets.
#[derive(Error, Debug)]
pub enum ExtraDataError {
    /// Failed to parse a JSON data file.
    #[error("Failed to parse {file}: {source}")]
    ParseError {
        /// The file that failed to parse.
        file: &'static str,
        /// The underlying JSON parsing error.
        #[source]
        source: serde_json::Error,
    },

    /// A required localized string key was not found.
    #[error("Localized string not found: {key}")]
    LocalizedStringNotFound {
        /// The key that was not found.
        key: String,
    },
}

/// Errors that can occur during log parsing.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    /// The parser accumulated too many lines without a successful parse.
    #[error("Buffer overflow: accumulated {lines} lines without successful parse")]
    BufferOverflow {
        /// The number of lines accumulated before overflow.
        lines: usize,
    },

    /// A parser rule reached an unreachable state (indicates a bug).
    #[error("Unreachable parse state in rule: {rule}")]
    UnreachableState {
        /// The name of the rule that reached an unreachable state.
        rule: &'static str,
    },

    /// Failed to parse an expected value.
    #[error("Failed to parse {field} in {rule}: {value}")]
    ParseValueError {
        /// The rule where the error occurred.
        rule: &'static str,
        /// The field that failed to parse.
        field: &'static str,
        /// The value that couldn't be parsed.
        value: String,
    },

    /// The input format was invalid.
    #[error("Invalid format in {rule}: {message}")]
    InvalidFormat {
        /// The rule where the error occurred.
        rule: &'static str,
        /// Description of the format error.
        message: String,
    },
}

/// Errors that can occur during file watching.
#[derive(Error, Debug)]
pub enum WatcherError {
    /// Failed to initialize the file watcher.
    #[error("Failed to initialize file watcher: {0}")]
    InitError(#[from] notify::Error),

    /// Failed to read the log file.
    #[error("Failed to read log file {path}: {source}")]
    ReadError {
        /// The path to the file that couldn't be read.
        path: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to seek in the log file.
    #[error("Failed to seek in log file {path}: {source}")]
    SeekError {
        /// The path to the file.
        path: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The watch directory does not exist.
    #[error("Watch directory does not exist: {path}")]
    DirectoryNotFound {
        /// The path that doesn't exist.
        path: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_error_display() {
        let err = ParserError::BufferOverflow { lines: 100 };
        assert_eq!(
            err.to_string(),
            "Buffer overflow: accumulated 100 lines without successful parse"
        );
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::MissingEnvVar { name: "LOG_DIR" };
        assert_eq!(err.to_string(), "Missing environment variable: LOG_DIR");
    }
}
