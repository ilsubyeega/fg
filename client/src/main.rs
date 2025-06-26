use std::env;

use temporal_rs::options::ToStringRoundingOptions;
use tracing::{Level, debug};

mod extra_data;
mod models;
mod parser;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    tracing_subscriber::fmt()
        .with_target(true)
        .with_max_level(Level::DEBUG)
        .init();

    let log_dir = &env::var("LOG_DIR").unwrap();
    let log_file = &env::var("LOG_FILE").unwrap();
    let file_path = format!("{}/{}", log_dir, log_file);

    let watch_rx = parser::task_watch::watch_dir(log_dir, log_file).await;
    let reader_rx = parser::task_watch::read_log_file(watch_rx, &file_path).await;
    let mut parser_rx = parser::task_parser::parse_from_str_rx(reader_rx).await;
    while let Some((message, instant)) = parser_rx.recv().await {
        debug!(
            "{:?}, {:?}",
            instant.and_then(|i| i
                .to_ixdtf_string(None, ToStringRoundingOptions::default())
                .ok()),
            message
        );
    }
}
