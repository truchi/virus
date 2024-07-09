use crate::editor::Editor;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::{
    process::ChildStdin,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        Mutex as IoMutex,
    },
};
use virus_lsp::{LspClient, LspClients};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       AsyncActorFunction                                       //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type AsyncActorFunction = Box<
    dyn FnOnce(
            Arc<IoMutex<LspClient<ChildStdin>>>,
            Arc<Mutex<Editor>>,
        ) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send,
>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        AsyncActorSender                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type AsyncActorSender = UnboundedSender<AsyncActorFunction>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           AsyncActor                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type AsyncActorReceiver = UnboundedReceiver<AsyncActorFunction>;

pub struct AsyncActor {
    editor: Arc<Mutex<Editor>>,
    clients: LspClients,
    receiver: AsyncActorReceiver,
}

impl AsyncActor {
    pub fn new(
        editor: Arc<Mutex<Editor>>,
        clients: LspClients,
        receiver: AsyncActorReceiver,
    ) -> Self {
        Self {
            editor,
            clients,
            receiver,
        }
    }

    pub async fn run(mut self) {
        while let Some(function) = self.receiver.recv().await {
            let editor = self.editor.clone();
            let rust_lsp_client = self.clients.rust();

            tokio::spawn(async move {
                function(rust_lsp_client, editor).await;
            });
        }
    }
}
