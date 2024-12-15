use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Command                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

use Command::*;

#[derive(Debug)]
enum Command {
    Watch(PathBuf),
    Unwatch(PathBuf),
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          WatcherEvent                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type WatcherEvent = notify::Event;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         WatcherHandle                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct WatcherClient(tokio::sync::mpsc::UnboundedSender<Command>);

impl WatcherClient {
    pub fn watch(&self, path: PathBuf) {
        self.0.send(Watch(path)).expect("send command");
    }

    pub fn unwatch(&self, path: PathBuf) {
        self.0.send(Unwatch(path)).expect("send command");
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          WatcherActor                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct WatcherActor {
    watcher: RecommendedWatcher,
    command_receiver: tokio::sync::mpsc::UnboundedReceiver<Command>,
    event_receiver: tokio::sync::mpsc::UnboundedReceiver<WatcherEvent>,
}

impl WatcherActor {
    pub fn new() -> (WatcherClient, Self) {
        let (command_sender, command_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();

        (
            WatcherClient(command_sender),
            Self {
                watcher: recommended_watcher(move |event| match event {
                    Ok(event) => event_sender.send(event).expect("send event"),
                    Err(err) => debug_assert!(false, "{err:#?}"),
                })
                .expect("recommended watcher"),
                command_receiver,
                event_receiver,
            },
        )
    }

    pub async fn run<F>(&mut self, mut handler: F)
    where
        F: FnMut(WatcherEvent),
    {
        loop {
            tokio::select! {
                command = self.command_receiver.recv() => {
                    match command {
                        Some(Watch(path)) => self.watch(&path),
                        Some(Unwatch(path)) => self.unwatch(&path),
                        None => break,
                    }
                },
                event = self.event_receiver.recv() => handler(event.expect("receive event")),
            };
        }
    }
}

/// Private.
impl WatcherActor {
    fn watch(&mut self, path: &Path) {
        self.watcher
            .watch(path, RecursiveMode::NonRecursive)
            .expect("watch");
    }

    fn unwatch(&mut self, path: &Path) {
        self.watcher.unwatch(path).expect("unwatch");
    }
}
