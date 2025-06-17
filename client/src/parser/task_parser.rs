use temporal_rs::{Instant, PlainTime, Temporal};
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

/// Ensures the log time does not differ between time ranges.
pub fn try_parse_log_time(log: &str) -> Option<Instant> {
    let re = create_regex(
        r"(?<hours>[0-9]+):(?<minutes>[0-9]+):(?<seconds>[0-9]+).(?<milliseconds>[0-9]+):",
    );

    for line in log.lines() {
        let Some(caps) = re.captures(line) else {
            continue;
        };
        // 09:14:00.422
        let hours = caps.name("hours").unwrap().as_str().parse::<u8>().unwrap();
        let minutes = caps
            .name("minutes")
            .unwrap()
            .as_str()
            .parse::<u8>()
            .unwrap();
        let seconds = caps
            .name("seconds")
            .unwrap()
            .as_str()
            .parse::<u8>()
            .unwrap();
        let milliseconds = caps
            .name("milliseconds")
            .unwrap()
            .as_str()
            .parse::<u16>()
            .unwrap();

        // Let temporal-rs handle the timezone information.
        let zoned_date_time = Temporal::now().zoned_date_time_iso(None).unwrap();
        let Ok(time) = PlainTime::new(hours, minutes, seconds, milliseconds, 0, 0) else {
            warn!(
                "Could not create PlainTime which is {}:{}:{}:{}",
                hours, minutes, seconds, milliseconds
            );
            continue;
        };
        zoned_date_time.with_plain_time(Some(time.clone())).unwrap();
        let Ok(zoned_date_time) = zoned_date_time.with_plain_time(Some(time)) else {
            warn!(
                "Could not call ZonedDateTime::with_plain_time which is {}:{}:{}:{}",
                hours, minutes, seconds, milliseconds
            );
            continue;
        };

        let instant = zoned_date_time.to_instant();
        return Some(instant);
    }

    None
}
