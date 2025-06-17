use std::{backtrace::Backtrace, env, panic};

use tracing::error;

mod extra_data;
mod models;
mod parser;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // Workaround for tokio panic handling.
    // https://github.com/tokio-rs/tokio/issues/2002#issuecomment-1020443386
    panic::set_hook(Box::new(|info| {
        error!("Got panic. @info:{}", info);
        error!("Backtrace: {}", Backtrace::force_capture());
        std::process::abort();
    }));
    dotenvy::dotenv().unwrap();
    let log_dir = &env::var("LOG_DIR").unwrap();
    let log_file = &env::var("LOG_FILE").unwrap();
    let file_path = format!("{}/{}", log_dir, log_file);

    let watch_rx = parser::task_watch::watch_dir(log_dir, log_file).await;
    let reader_rx = parser::task_watch::read_log_file(watch_rx, &file_path).await;
    let mut parser_rx = parser::task_parser::parse_from_str_rx(reader_rx).await;
    while let Some(message) = parser_rx.recv().await {
        println!("{:?}", message);
    }
}
