use crate::editor::Editor;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::{process::ChildStdin, sync::mpsc::UnboundedReceiver, sync::Mutex as IoMutex};
use virus_lsp::{LspClient, LspClients};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       AsyncActorMessage                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct AsyncActorMessage {
    pub f: Box<
        dyn FnMut(
                Arc<IoMutex<LspClient<ChildStdin>>>,
                Arc<Mutex<Editor>>,
            ) -> Pin<Box<dyn Future<Output = ()> + Send>>
            + Send,
    >,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           AsyncActor                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct AsyncActor {
    editor: Arc<Mutex<Editor>>,
    clients: LspClients,
    receiver: UnboundedReceiver<AsyncActorMessage>,
}

impl AsyncActor {
    pub fn new(
        editor: Arc<Mutex<Editor>>,
        clients: LspClients,
        receiver: UnboundedReceiver<AsyncActorMessage>,
    ) -> Self {
        Self {
            editor,
            clients,
            receiver,
        }
    }

    pub async fn run(mut self) {
        while let Some(mut message) = self.receiver.recv().await {
            println!("Async actor received a message!");

            let editor = self.editor.clone();
            let rust_lsp_client = self.clients.rust();

            tokio::spawn(async move {
                println!("Async actor calling async function!");
                (message.f)(rust_lsp_client, editor).await;
            });
        }
    }
}
