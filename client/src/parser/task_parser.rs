use tokio::sync::mpsc::{self, Receiver};

use crate::models::messages::FGGameMessage;

use super::{ParseResult, rules::FGParserRule};

pub async fn parse_from_str_rx(mut str_rx: Receiver<String>) -> Receiver<FGGameMessage> {
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

            for rule in rules {
                let eval = rule(&str);
                match eval {
                    ParseResult::Parsed(data) => {
                        tx.send(data).await.unwrap();
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
                    _ => {
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
