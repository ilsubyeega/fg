use notify::{
    Config, EventKind, RecommendedWatcher, Watcher,
    event::{AccessKind, CreateKind, ModifyKind},
};
use std::io::SeekFrom;
use tokio::{
    fs::OpenOptions,
    sync::mpsc::{self, Receiver},
};

use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};

/*
* Fallguys IO state on Linux/Steam/Proton.

* - Access(Open(Any))
* - Access(Close(Write))
* - Access(Open(Any))
* - Modify(Name(From))
* - Modify(Name(To))
* - Modify(Name(Both)) -> has 2 paths vector, before then new.
* - Access(Open(Any))
* - Create(File)
* - Modify(Data(Any))
* - Access(Close(Write))
*/

/// Creates a async watcher in sync runtime.
pub fn async_watcher()
-> notify::Result<(RecommendedWatcher, Receiver<notify::Result<notify::Event>>)> {
    let (tx, rx) = mpsc::channel(1024);
    let watcher = RecommendedWatcher::new(
        move |res| {
            tx.blocking_send(res).unwrap();
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

pub async fn watch_dir(dir_path: &str, file_name: &str) -> Receiver<WatchMessage> {
    let (tx, rx) = mpsc::channel(1024);
    let (mut watcher, mut watch_rx) = async_watcher().unwrap();

    watcher
        .watch(dir_path.as_ref(), notify::RecursiveMode::NonRecursive)
        .unwrap();

    let file_path = format!("{}/{}", dir_path, file_name);
    let dir_path = dir_path.to_owned();
    let file_name = file_name.to_owned();
    tokio::spawn(async move {
        while let Some(event) = watch_rx.recv().await {
            // Just panic if event is Err.
            let event = event.unwrap();

            if event.need_rescan() {
                // Re-watch if required.
                watcher
                    .watch(dir_path.as_ref(), notify::RecursiveMode::NonRecursive)
                    .unwrap();
                println!("Rescanning")
            }

            if let Some(path) = event.paths.first() {
                if path.ends_with(&file_name) {
                    let msg = match event.kind {
                        EventKind::Access(AccessKind::Close(_)) => Some(WatchMessage::Closed),
                        EventKind::Create(CreateKind::File) => Some(WatchMessage::FileCreated),
                        EventKind::Modify(ModifyKind::Data(_)) => {
                            let file = OpenOptions::new()
                                .read(true)
                                .open(&file_path)
                                .await
                                .unwrap();
                            let file_len = file.metadata().await.unwrap().len();
                            Some(WatchMessage::ContentModified { length: file_len })
                        }
                        _ => None,
                    };

                    if let Some(msg) = msg {
                        tx.send(msg).await.unwrap();
                    }
                }
            }
        }
    });

    rx
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchMessage {
    FileCreated,
    ContentModified { length: u64 },
    Closed,
}

/// Reads the log file and then stream into single line.
pub async fn read_log_file(
    mut watch_rx: Receiver<WatchMessage>,
    file_path: &str,
) -> Receiver<String> {
    let (tx, rx) = mpsc::channel(1024);

    let file_path = file_path.to_owned();
    tokio::spawn(async move {
        let mut buffer = 0;

        while let Some(watch_msg) = watch_rx.recv().await {
            if watch_msg == WatchMessage::FileCreated {
                buffer = 0;
            }

            if let WatchMessage::ContentModified { length } = watch_msg {
                // file length is less than buffer size
                if length < buffer {
                    unreachable!(
                        "The log file is less than the buffer size. Maybe the file content has been changed? It should be not happened."
                    );
                }
                let mut file = OpenOptions::new()
                    .read(true)
                    .open(&file_path)
                    .await
                    .unwrap();
                file.seek(SeekFrom::Start(buffer)).await.unwrap();
                let mut lines = BufReader::new(file).lines();

                while let Some(line) = lines.next_line().await.unwrap() {
                    tx.send(line).await.unwrap();
                }

                buffer = length;
            }
        }
    });

    rx
}
