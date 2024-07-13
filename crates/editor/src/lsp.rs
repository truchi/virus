use crate::{async_actor::AsyncActorSender, document::Document, editor::Editor};
use serde_json::Value;
use std::{
    future::Future,
    path::Component,
    sync::{Mutex, Weak},
};
use tokio::sync::oneshot::Sender;
use virus_lsp::{
    enumerations::{PositionEncodingKind, TraceValues},
    structures::{
        ClientCapabilities, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, GeneralClientCapabilities, InitializeParams,
        InitializeParamsProcessId, InitializeParamsWorkspaceFolders, InitializedParams,
        TextDocumentIdentifier, TextDocumentItem, VersionedTextDocumentIdentifier,
        WindowClientCapabilities, WorkDoneProgressParams, WorkspaceFolder,
    },
    type_aliases::{
        ProgressToken, TextDocumentContentChangeEvent, TextDocumentContentChangeEventRangeAndText,
    },
    Integer, ServerMessage, ServerMessageReceiver, ServerNotification, ServerRequest,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Lsp                                               //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone)]
pub struct Lsp {
    async_actor: AsyncActorSender,
}

impl Lsp {
    pub fn new(async_actor: AsyncActorSender) -> Self {
        Self { async_actor }
    }

    pub fn init(&self, rust_receiver: ServerMessageReceiver) {
        let process_id = std::process::id() as Integer;

        self.send(|editor| Self::rust_lsp_handler(editor, rust_receiver));
        self.send(move |editor| async move {
            let (mut client, folder) = {
                let editor = editor.upgrade().unwrap();
                let mut editor = editor.lock().unwrap();
                let root = editor.root();
                let folder = WorkspaceFolder {
                    uri: format!("file://{}", root.to_owned().to_str().unwrap()),
                    name: match root.components().last() {
                        Some(Component::Normal(str)) => str.to_str().unwrap().to_owned(),
                        _ => panic!(),
                    },
                };

                (editor.lsps.rust(), folder)
            };

            let result = client
                .request()
                .initialize(initialize_params(
                    process_id,
                    String::from("Initialize work done progress token"),
                    Some(serde_json::json!({
                        "rustfmt": {
                            "rangeFormatting": {
                                "enable": true, // Requires nightly...
                            },
                        },
                    })),
                    folder,
                ))
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
            client
                .notification()
                .initialized(InitializedParams {})
                .await
                .unwrap();
            client.wait_for_work_done().await;
            client.init(result);
        });
    }

    pub fn open_document(&self, document: &Document) {
        let uri = document.path().as_os_str().to_str().unwrap();
        let uri = format!("file://{uri}");
        let language_id = document.path().extension().unwrap().to_str().unwrap();
        let language_id = (language_id == "rs")
            .then(|| "rust")
            .unwrap_or(language_id)
            .to_owned();
        let text = document.rope().to_string();
        let version = document.version() as Integer;

        self.send(move |editor| async move {
            let mut client = {
                let editor = editor.upgrade().unwrap();
                let mut editor = editor.lock().unwrap();

                editor.lsps.rust()
            };

            client.inited().await;
            client
                .notification()
                .text_document_did_open(DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri,
                        language_id,
                        version,
                        text,
                    },
                })
                .await
                .unwrap();
        });
    }

    pub fn change_document(
        &self,
        document: &Document,
        changes: impl IntoIterator<Item = TextDocumentContentChangeEventRangeAndText> + Send + 'static,
    ) {
        let uri = document.path().as_os_str().to_str().unwrap();
        let uri = format!("file://{uri}");
        let version = document.version() as Integer;

        self.send(move |editor| async move {
            let mut client = {
                let editor = editor.upgrade().unwrap();
                let mut editor = editor.lock().unwrap();

                editor.lsps.rust()
            };

            client.inited().await;
            client
                .notification()
                .text_document_did_change(DidChangeTextDocumentParams {
                    text_document: VersionedTextDocumentIdentifier {
                        text_document_identifier: TextDocumentIdentifier { uri },
                        version,
                    },
                    content_changes: changes
                        .into_iter()
                        .map(TextDocumentContentChangeEvent::RangeAndText)
                        .collect(),
                })
                .await
                .unwrap();
        });
    }

    pub fn close_document(self, document: &Document) {
        let uri = document.path().as_os_str().to_str().unwrap();
        let uri = format!("file://{uri}");

        self.send(|editor| async move {
            let mut client = {
                let editor = editor.upgrade().unwrap();
                let mut editor = editor.lock().unwrap();

                editor.lsps.rust()
            };

            client.inited().await;
            client
                .notification()
                .text_document_did_close(DidCloseTextDocumentParams {
                    text_document: TextDocumentIdentifier { uri },
                })
                .await
                .unwrap();
        });
    }

    pub fn exit(&self, sender: Sender<()>) {
        self.send(|editor| async move {
            let client = {
                let editor = editor.upgrade().unwrap();
                let editor = editor.lock().unwrap();

                editor.lsps.rust_if_spawned()
            };

            if let Some(mut client) = client {
                client
                    .request()
                    .shutdown()
                    .await
                    .unwrap()
                    .await
                    .unwrap()
                    .unwrap();
                client.notification().exit().await.unwrap();
            }

            sender.send(()).unwrap();
        });
    }
}

/// Private.
impl Lsp {
    fn send<F, Fut>(&self, function: F)
    where
        F: 'static + Send + FnOnce(Weak<Mutex<Editor>>) -> Fut,
        Fut: 'static + Send + Future<Output = ()>,
    {
        self.async_actor
            .send(Box::new(|editor| Box::pin(function(editor))))
            .expect("Failed to send to async actor");
    }

    async fn rust_lsp_handler(_editor: Weak<Mutex<Editor>>, mut receiver: ServerMessageReceiver) {
        while let Some(message) = receiver.recv().await {
            match message {
                ServerMessage::ServerNotification(notification) => {
                    // dbg!(&notification);

                    match notification {
                        ServerNotification::CancelRequest(_) => {}
                        ServerNotification::LogTrace(trace) => {
                            dbg!(trace);
                        }
                        ServerNotification::Progress(_) => {}
                        ServerNotification::TelemetryEvent(_) => {}
                        ServerNotification::TextDocumentPublishDiagnostics(_) => {}
                        ServerNotification::WindowLogMessage(_) => {}
                        ServerNotification::WindowShowMessage(_) => {}
                    }
                }
                ServerMessage::ServerRequest(request) => {
                    // dbg!(&request);

                    match request {
                        ServerRequest::ClientRegisterCapability(_, _) => {}
                        ServerRequest::ClientUnregisterCapability(_, _) => {}
                        ServerRequest::WindowShowDocument(_, _) => {}
                        ServerRequest::WindowShowMessageRequest(_, _) => {}
                        ServerRequest::WindowWorkDoneProgressCreate(_, _) => {}
                        ServerRequest::WorkspaceApplyEdit(_, _) => {}
                        ServerRequest::WorkspaceCodeLensRefresh(_) => {}
                        ServerRequest::WorkspaceConfiguration(_, _) => {}
                        ServerRequest::WorkspaceDiagnosticRefresh(_) => {}
                        ServerRequest::WorkspaceInlayHintRefresh(_) => {}
                        ServerRequest::WorkspaceInlineValueRefresh(_) => {}
                        ServerRequest::WorkspaceSemanticTokensRefresh(_) => {}
                        ServerRequest::WorkspaceWorkspaceFolders(_) => {}
                    }
                }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

fn initialize_params(
    process_id: Integer,
    work_done_token: String,
    options: Option<Value>,
    folder: WorkspaceFolder,
) -> InitializeParams {
    InitializeParams {
        work_done_progress_params: WorkDoneProgressParams {
            work_done_token: Some(ProgressToken::String(work_done_token)),
        },
        process_id: InitializeParamsProcessId::Integer(process_id),
        client_info: None,
        locale: None,
        capabilities: ClientCapabilities {
            workspace: None,
            text_document: None,
            notebook_document: None,
            window: Some(WindowClientCapabilities {
                work_done_progress: Some(true),
                show_message: None,
                show_document: None,
            }),
            general: Some(GeneralClientCapabilities {
                stale_request_support: None,
                regular_expressions: None,
                markdown: None,
                position_encodings: Some(vec![PositionEncodingKind::Utf8]),
            }),
            experimental: None,
        },
        initialization_options: options.map(|options| serde_json::from_value(options).unwrap()),
        trace: Some(TraceValues::Messages),
        workspace_folders: Some(InitializeParamsWorkspaceFolders::WorkspaceFolderList(vec![
            folder,
        ])),
    }
}
