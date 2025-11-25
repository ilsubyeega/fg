//! Log line parsing task.
//!
//! This module provides the async task that processes raw log lines
//! and converts them into structured game messages.
//!
//! # Architecture
//!
//! The parser task receives strings from a channel (typically from
//! [`task_watch`](super::task_watch)), applies parsing rules, and
//! outputs parsed messages to another channel.
//!
//! # Multi-line Handling
//!
//! Some log messages span multiple lines. The parser maintains state
//! to buffer lines when a rule indicates it needs more input.

use std::sync::LazyLock;

use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::map_res,
};
use temporal_rs::{DateDuration, Duration, Instant, PlainTime, Temporal, TimeZone};
use tokio::sync::mpsc::{self, Receiver};
use tracing::warn;

use crate::models::messages::FGGameMessage;

use super::{ParseResult, rules::FGParserRule};

/// Maximum number of lines to buffer for multi-line parsing.
const MAX_BUFFER_LINES: usize = 100;

/// Processes log lines and emits parsed game messages.
///
/// This async task:
/// 1. Receives raw log lines from `str_rx`
/// 2. Applies parsing rules to extract game events
/// 3. Handles multi-line messages by buffering
/// 4. Sends parsed messages with timestamps to the returned channel
///
/// # Arguments
///
/// * `str_rx` - Channel receiving raw log lines
///
/// # Returns
///
/// A channel that yields `(FGGameMessage, Option<Instant>)` tuples,
/// where the instant is the parsed timestamp from the log line.
pub async fn parse_from_str_rx(
    mut str_rx: Receiver<String>,
) -> Receiver<(FGGameMessage, Option<Instant>)> {
    let (tx, rx) = mpsc::channel(1024);

    tokio::spawn(async move {
        let mut need_more_lines_rule: Option<FGParserRule> = None;
        let mut temp_buffer: String = String::new();
        
        while let Some(mut str) = str_rx.recv().await {
            let rules = if need_more_lines_rule.is_some() {
                // Append to buffer for multi-line parsing
                let mut buf = temp_buffer;
                buf.push('\n');
                buf.push_str(&str);
                temp_buffer = buf.clone();
                str = buf;

                vec![need_more_lines_rule.clone().unwrap()]
            } else {
                super::rules::rules()
            };

            // Safety check: don't let buffer grow unbounded
            if temp_buffer.lines().count() > MAX_BUFFER_LINES {
                warn!("Parser buffer exceeded {} lines, resetting", MAX_BUFFER_LINES);
                need_more_lines_rule = None;
                temp_buffer = String::new();
                continue;
            }

            for rule in rules {
                let eval = rule(&str);
                match eval {
                    ParseResult::Parsed(data) => {
                        tx.send((data, try_parse_log_time(&str))).await.unwrap();
                        need_more_lines_rule = None;
                        temp_buffer = String::new();
                        break;
                    }
                    ParseResult::NeedMoreLines => {
                        need_more_lines_rule = Some(rule);
                        temp_buffer = str;
                        break;
                    }
                    #[allow(deprecated)]
                    ParseResult::Unreachable => {
                        warn!("Parser reached unreachable state, continuing");
                        need_more_lines_rule = None;
                        temp_buffer = String::new();
                    }
                    ParseResult::None => {
                        if need_more_lines_rule.is_some() || !temp_buffer.is_empty() {
                            need_more_lines_rule = None;
                            temp_buffer = String::new();
                        }
                    }
                }
            }
        }
    });
    rx
}

// =============================================================================
// Time Parsing
// =============================================================================

static UTC: LazyLock<TimeZone> = LazyLock::new(|| {
    TimeZone::try_from_identifier_str("Z").expect("Could not create UTC TimeZone")
});

static DAY_DURATION: LazyLock<Duration> = LazyLock::new(|| {
    DateDuration::new(0, 0, 0, 1)
        .expect("Cannot create duration.")
        .into()
});

/// Parses time components from a log line: "HH:MM:SS.mmm:"
fn parse_log_time(input: &str) -> IResult<&str, (u8, u8, u8, u16)> {
    // In nom 8, tuples implement Parser directly, so we can use .parse()
    let (input, (hours, _, minutes, _, seconds, _, millis, _)) = (
        map_res(digit1, |s: &str| s.parse::<u8>()),
        char(':'),
        map_res(digit1, |s: &str| s.parse::<u8>()),
        char(':'),
        map_res(digit1, |s: &str| s.parse::<u8>()),
        char('.'),
        map_res(digit1, |s: &str| s.parse::<u16>()),
        tag(":"),
    ).parse(input)?;
    
    Ok((input, (hours, minutes, seconds, millis)))
}

/// Attempts to parse the timestamp from a log line.
///
/// Fall Guys log lines start with a timestamp in the format:
/// `HH:MM:SS.mmm: [Component] Message`
///
/// This function extracts that timestamp and converts it to an [`Instant`].
///
/// # Arguments
///
/// * `log` - The log line(s) to parse
///
/// # Returns
///
/// `Some(Instant)` if a valid timestamp was found, `None` otherwise.
pub fn try_parse_log_time(log: &str) -> Option<Instant> {
    return None;
    // FIXME: Temporarily disabled time parsing. This throws IO error.

    let zoned_datetime = Temporal::now()
        .zoned_date_time_iso(Some(UTC.clone()))
        .expect("Cannot get current zoned datetime in UTC");
    
    for line in log.lines() {
        // Try to parse the timestamp at the start of the line
        let Ok((_, (hours, minutes, seconds, milliseconds))) = parse_log_time(line) else {
            continue;
        };

        let Ok(plain_time) = PlainTime::new(hours, minutes, seconds, milliseconds, 0, 0) else {
            warn!(
                "Could not create PlainTime: {:02}:{:02}:{:02}.{:03}",
                hours, minutes, seconds, milliseconds
            );
            continue;
        };

        let hours_current = zoned_datetime
            .hour()
            .expect("Cannot get current hour from zoned_datetime");
        
        // Handle day rollover: if log time > current time, assume it's from yesterday
        let zoned_datetime = if hours > hours_current {
            zoned_datetime
                .subtract(&DAY_DURATION, None)
                .and_then(|d| d.with_plain_time(Some(plain_time)))
                .expect("Cannot modify zoned_datetime with plain time, -24h")
        } else {
            zoned_datetime
                .with_plain_time(Some(plain_time))
                .expect("Cannot modify zoned_datetime with plain time")
        };

        return Some(zoned_datetime.to_instant());
    }

    None
}

// =============================================================================
// Types
// =============================================================================

/// Represents the state of the parser between iterations.
#[derive(Debug, Clone)]
pub enum ParserState {
    /// Ready to receive a new line.
    NewContent,
    /// Waiting for additional lines to complete a multi-line message.
    RequiresMoreLine(String),
    /// A line ready to be parsed.
    ParseLine(String),
    /// Multiple lines ready to be parsed.
    ParseLines(Vec<String>),
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_time() {
        let (_, (h, m, s, ms)) = parse_log_time("09:14:00.422: [GameStateMachine]").unwrap();
        assert_eq!(h, 9);
        assert_eq!(m, 14);
        assert_eq!(s, 0);
        assert_eq!(ms, 422);
    }

    #[test]
    fn test_parse_log_time_different_values() {
        let (_, (h, m, s, ms)) = parse_log_time("23:59:59.999: Message").unwrap();
        assert_eq!(h, 23);
        assert_eq!(m, 59);
        assert_eq!(s, 59);
        assert_eq!(ms, 999);
    }
}
