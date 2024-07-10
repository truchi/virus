use crate::editor::Editor;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       AsyncActorFunction                                       //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type AsyncActorSender = UnboundedSender<AsyncActorFunction>;
pub type AsyncActorReceiver = UnboundedReceiver<AsyncActorFunction>;

pub type AsyncActorFunction =
    Box<dyn FnOnce(Arc<Mutex<Editor>>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           AsyncActor                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct AsyncActor {
    editor: Arc<Mutex<Editor>>,
    receiver: AsyncActorReceiver,
}

impl AsyncActor {
    pub fn new(editor: Arc<Mutex<Editor>>, receiver: AsyncActorReceiver) -> Self {
        Self { editor, receiver }
    }

    pub async fn run(mut self) {
        while let Some(function) = self.receiver.recv().await {
            let editor = self.editor.clone();

            tokio::spawn(async move { function(editor).await });
        }
    }
}
