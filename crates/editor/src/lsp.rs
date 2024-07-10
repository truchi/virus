use crate::editor::Editor;
use serde_json::Value;
use std::{
    path::Component,
    sync::{Arc, Mutex},
};
use virus_lsp::{
    enumerations::{PositionEncodingKind, TraceValues},
    structures::{
        ClientCapabilities, GeneralClientCapabilities, InitializeParams, InitializeParamsProcessId,
        InitializeParamsWorkspaceFolders, InitializedParams, WindowClientCapabilities,
        WorkDoneProgressParams, WorkspaceFolder,
    },
    type_aliases::ProgressToken,
    ServerMessage, ServerMessageReceiver, ServerNotification, ServerRequest,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Lsp                                               //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

const ASYNC_ACTOR_SEND_FAIL: &'static str = "Failed to send to async actor";

pub struct Lsp<'editor> {
    pub editor: &'editor mut Editor,
}

impl<'editor> Lsp<'editor> {
    pub fn init(self, rust_receiver: ServerMessageReceiver) -> Self {
        let process_id = std::process::id() as i32;

        self.editor
            .async_actor
            .send(Box::new(|editor| {
                Box::pin(rust_lsp_handler(editor, rust_receiver))
            }))
            .expect(ASYNC_ACTOR_SEND_FAIL);

        self.editor
            .async_actor
            .send(Box::new(move |editor| {
                Box::pin(async move {
                    let (client, folder) = {
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

                    let mut client = client.lock().await;
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
                })
            }))
            .expect(ASYNC_ACTOR_SEND_FAIL);

        self
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

async fn rust_lsp_handler(_editor: Arc<Mutex<Editor>>, mut receiver: ServerMessageReceiver) {
    while let Some(message) = receiver.recv().await {
        match message {
            ServerMessage::ServerNotification(notification) => match notification {
                ServerNotification::CancelRequest(_) => {}
                ServerNotification::LogTrace(trace) => {
                    dbg!(trace);
                }
                ServerNotification::Progress(_) => {}
                ServerNotification::TelemetryEvent(_) => {}
                ServerNotification::TextDocumentPublishDiagnostics(_) => {}
                ServerNotification::WindowLogMessage(_) => {}
                ServerNotification::WindowShowMessage(_) => {}
            },
            ServerMessage::ServerRequest(request) => match request {
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
            },
        }
    }
}

fn initialize_params(
    process_id: i32,
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
