use std::sync::LazyLock;

use temporal_rs::{DateDuration, Duration, Instant, PlainTime, Temporal, TimeZone};
use tokio::sync::mpsc::{self, Receiver};
use tracing::warn;

use crate::models::messages::FGGameMessage;

use super::{ParseResult, create_regex, rules::FGParserRule};

pub async fn parse_from_str_rx(
    mut str_rx: Receiver<String>,
) -> Receiver<(FGGameMessage, Option<Instant>)> {
    let (tx, rx) = mpsc::channel(1024);

    tokio::spawn(async move {
        let mut need_more_lines_rule: Option<FGParserRule> = None;
        let mut temp_buffer: String = String::new();
        while let Some(mut str) = str_rx.recv().await {
            let rules = if need_more_lines_rule.is_some() {
                let mut buf = temp_buffer;
                buf.push('\n');
                buf.push_str(&str);
                temp_buffer = buf.clone();
                str = buf;

                vec![need_more_lines_rule.clone().unwrap()]
            } else {
                super::rules::rules()
            };

            if temp_buffer.lines().count() > 100 {
                unreachable!("Parser just read 100 lines of buffer, which should not happen.");
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
                    ParseResult::Unreachable => {
                        println!("Unreachable state reached");
                        unreachable!();
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

static UTC: LazyLock<TimeZone> = LazyLock::new(|| {
    TimeZone::try_from_identifier_str("Z").expect("Could not create UTC TimeZone")
});
static DAY_DURATION: LazyLock<Duration> = LazyLock::new(|| {
    DateDuration::new(0, 0, 0, 1)
        .expect("Cannot create duration.")
        .into()
});

/// Ensures the log time does not differ between time ranges.
pub fn try_parse_log_time(log: &str) -> Option<Instant> {
    let re = create_regex(
        r"(?<hours>[0-9]+):(?<minutes>[0-9]+):(?<seconds>[0-9]+).(?<milliseconds>[0-9]+):",
    );

    let zoned_datetime = Temporal::now()
        .zoned_date_time_iso(Some(UTC.clone()))
        .expect("Cannot get current zoned datetime in UTC");
    for line in log.lines() {
        let Some(caps) = re.captures(line) else {
            continue;
        };
        // 09:14:00.422
        let hours = caps
            .name("hours")
            .and_then(|m| m.as_str().parse::<u8>().ok())
            .expect("Cannot parse from regex capture.");
        let minutes = caps
            .name("minutes")
            .and_then(|m| m.as_str().parse::<u8>().ok())
            .expect("Cannot parse from regex capture.");
        let seconds = caps
            .name("seconds")
            .and_then(|m| m.as_str().parse::<u8>().ok())
            .expect("Cannot parse from regex capture.");
        let milliseconds = caps
            .name("milliseconds")
            .and_then(|m| m.as_str().parse::<u16>().ok())
            .expect("Cannot parse from regex capture.");

        let Ok(plain_time) = PlainTime::new(hours, minutes, seconds, milliseconds, 0, 0) else {
            warn!(
                "Could not create PlainTime which is {}:{}:{}:{}",
                hours, minutes, seconds, milliseconds
            );
            continue;
        };

        let hours_current = zoned_datetime
            .hour()
            .expect("Cannot get current hour from zoned_datetime");
        let zoned_datetime = if hours > hours_current {
            // Subtract 24 hours if the hours are greater than the current day
            // to ensure the time is within the current day.
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

pub enum ParserState {
    /// Get new line from it.
    NewContent,
    /// Needs more line.
    RequiresMoreLine(String),
    /// Parse this line, thrown from rules.rs
    /// Usually from multiple-line parsing and then does not required for that.
    ParseLine(String),
    /// Vector of ParseLine
    ParseLines(Vec<String>),
}
