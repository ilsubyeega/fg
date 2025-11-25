//! Log parsing module for Fall Guys game logs.
//!
//! This module provides the core parsing infrastructure for reading and
//! interpreting Fall Guys log files. It consists of several components:
//!
//! - [`ParseResult`]: The result type for all parsing operations
//! - [`rules`]: Individual parsing rules for different log message types
//! - [`combinators`]: Reusable nom parser combinators
//! - [`task_parser`]: Async task for processing log lines
//! - [`task_watch`]: File system watcher for log file changes
//!
//! # Architecture
//!
//! ```text
//! Log File → task_watch → task_parser → rules → FGGameMessage
//!                ↓              ↓
//!         WatchMessage    ParseResult
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use fg_daemon::parser::{task_watch, task_parser};
//!
//! let watch_rx = task_watch::watch_dir("/path/to/logs", "Player.log").await;
//! let reader_rx = task_watch::read_log_file(watch_rx, "/path/to/logs/Player.log").await;
//! let mut parser_rx = task_parser::parse_from_str_rx(reader_rx).await;
//!
//! while let Some((message, timestamp)) = parser_rx.recv().await {
//!     println!("Parsed: {:?}", message);
//! }
//! ```

pub mod combinators;
pub mod rules;
pub mod task_parser;
pub mod task_watch;

/// The result of attempting to parse a log line.
///
/// Parsing is a three-outcome operation:
/// - Success: The line was parsed into a message
/// - Need more: The line is part of a multi-line message
/// - None: This rule doesn't apply to this line
///
/// # Type Parameter
///
/// * `T` - The type of successfully parsed message (typically [`FGGameMessage`])
///
/// # Examples
///
/// ```rust
/// use fg_daemon::parser::ParseResult;
///
/// fn example_parser(input: &str) -> ParseResult<String> {
///     if input.contains("hello") {
///         ParseResult::Parsed("Found hello!".to_string())
///     } else if input.starts_with("multi") {
///         ParseResult::NeedMoreLines
///     } else {
///         ParseResult::None
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseResult<T> {
    /// Successfully parsed the input into a message.
    ///
    /// Contains the parsed message of type `T`.
    Parsed(T),

    /// The parser needs more lines to complete parsing.
    ///
    /// This is used for multi-line log messages where a single
    /// rule needs to accumulate several lines before producing
    /// a complete result.
    NeedMoreLines,

    /// The parser encountered an unreachable/invalid state.
    ///
    /// This typically indicates a bug in the parser logic and
    /// should be converted to an error in production code.
    #[deprecated(
        since = "0.1.0",
        note = "Use proper error handling instead of Unreachable"
    )]
    Unreachable,

    /// This rule does not apply to the input.
    ///
    /// The parser should try the next rule in the chain.
    None,
}

impl<T> ParseResult<T> {
    /// Returns `true` if the result is [`Parsed`](ParseResult::Parsed).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fg_daemon::parser::ParseResult;
    ///
    /// let result: ParseResult<i32> = ParseResult::Parsed(42);
    /// assert!(result.is_parsed());
    /// ```
    #[inline]
    pub fn is_parsed(&self) -> bool {
        matches!(self, ParseResult::Parsed(_))
    }

    /// Returns `true` if the result is [`None`](ParseResult::None).
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(self, ParseResult::None)
    }

    /// Returns `true` if the result is [`NeedMoreLines`](ParseResult::NeedMoreLines).
    #[inline]
    pub fn needs_more_lines(&self) -> bool {
        matches!(self, ParseResult::NeedMoreLines)
    }

    /// Converts the `ParseResult<T>` into an `Option<T>`.
    ///
    /// Returns `Some(value)` if this is `Parsed(value)`, otherwise `None`.
    #[inline]
    pub fn into_option(self) -> Option<T> {
        match self {
            ParseResult::Parsed(v) => Some(v),
            _ => None,
        }
    }

    /// Maps a `ParseResult<T>` to `ParseResult<U>` by applying a function.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> ParseResult<U> {
        match self {
            ParseResult::Parsed(v) => ParseResult::Parsed(f(v)),
            ParseResult::NeedMoreLines => ParseResult::NeedMoreLines,
            #[allow(deprecated)]
            ParseResult::Unreachable => ParseResult::Unreachable,
            ParseResult::None => ParseResult::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_result_is_parsed() {
        let result: ParseResult<i32> = ParseResult::Parsed(42);
        assert!(result.is_parsed());
        assert!(!result.is_none());
    }

    #[test]
    fn test_parse_result_into_option() {
        let parsed: ParseResult<i32> = ParseResult::Parsed(42);
        assert_eq!(parsed.into_option(), Some(42));

        let none: ParseResult<i32> = ParseResult::None;
        assert_eq!(none.into_option(), None);
    }

    #[test]
    fn test_parse_result_map() {
        let result: ParseResult<i32> = ParseResult::Parsed(42);
        let mapped = result.map(|x| x * 2);
        assert_eq!(mapped.into_option(), Some(84));
    }
}
