use lsp::{
    client::{Client, RequestOrNotification},
    types::{
        enumerations::PositionEncodingKind,
        notifications::{Exit, Initialized, Notification},
        requests::{
            Initialize, Request, Shutdown, TextDocumentDefinition, TextDocumentDefinitionResult,
            TextDocumentRangeFormatting, TextDocumentRangeFormattingResult,
        },
        structures::{
            ClientCapabilities, DefinitionParams, DocumentRangeFormattingParams, FormattingOptions,
            GeneralClientCapabilities, InitializeError, InitializeParams,
            InitializeParamsProcessId, InitializeParamsWorkspaceFolders, InitializeResult,
            InitializedParams, PartialResultParams, Position, Range, TextDocumentIdentifier,
            TextDocumentPositionParams, WindowClientCapabilities, WorkDoneProgressParams,
            WorkspaceFolder,
        },
        type_aliases::{LspAny, ProgressToken},
        Null,
    },
};
use std::{process::Stdio, time::Instant};
use tokio::{io::BufReader, process::Command};

#[tokio::main]
async fn main() {
    let mut child = Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let (stdin, stdout) = (child.stdin.take().unwrap(), child.stdout.take().unwrap());
    let (mut client, mut receiver) = Client::new(BufReader::new(stdout), stdin);
    let handle = tokio::spawn(async move {
        loop {
            match receiver.recv().await {
                Some(RequestOrNotification::Left(request)) => {
                    println!(
                        "{}: {:?}",
                        request.method,
                        request
                            .params
                            .as_ref()
                            .and_then(|params| params.as_object())
                            .and_then(|params| params.get("token"))
                            .and_then(|params| params.as_str()),
                    );
                }
                Some(RequestOrNotification::Right(notification)) => {
                    println!(
                        "{}: {:?} ({:?})",
                        notification.method,
                        notification
                            .params
                            .as_ref()
                            .and_then(|params| params.as_object())
                            .and_then(|params| params.get("token"))
                            .and_then(|params| params.as_str()),
                        notification
                            .params
                            .as_ref()
                            .and_then(|params| params.as_object())
                            .and_then(|params| params.get("value"))
                            .and_then(|params| params.as_object())
                            .and_then(|params| params.get("kind"))
                            .and_then(|params| params.as_str()),
                    );
                }
                None => {
                    println!("Stream done");
                    break;
                }
            }
        }
    });

    println!("Initialize");
    let options = serde_json::from_value::<LspAny>(serde_json::json!({
        "rustfmt": {
            "rangeFormatting": {
                "enable": true, // Requires nightly...
            },
        },
    }))
    .unwrap();
    let response = client
        .request::<InitializeResult, InitializeError, _>(
            Initialize::METHOD.into(),
            Some(InitializeParams {
                work_done_progress_params: WorkDoneProgressParams {
                    work_done_token: Some(ProgressToken::String(String::from(
                        "Initialize work done progress token",
                    ))),
                },
                process_id: InitializeParamsProcessId::Integer(std::process::id() as i32),
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
                initialization_options: Some(options.clone()),
                trace: None,
                workspace_folders: Some(InitializeParamsWorkspaceFolders::WorkspaceFolderList(
                    vec![WorkspaceFolder {
                        uri: String::from("file:///Users/romain/perso/virus"),
                        name: String::from("virus"),
                    }],
                )),
            }),
        )
        .await
        .unwrap()
        .await;
    dbg!(&response);

    println!("Initialized");
    client
        .notification(Initialized::METHOD.into(), Some(InitializedParams {}))
        .await
        .unwrap();

    wait(10).await;

    for _ in 0..0 {
        println!("TextDocumentDefinition");
        let response = client
            .request::<TextDocumentDefinitionResult, (), _>(
                TextDocumentDefinition::METHOD.into(),
                Some(DefinitionParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier {
                            uri: String::from(
                                "file:///Users/romain/perso/virus/crates/virus/src/main.rs",
                            ),
                        },
                        position: Position {
                            line: 8,
                            character: 11,
                        },
                    },
                    work_done_progress_params: WorkDoneProgressParams {
                        work_done_token: Some(ProgressToken::String(String::from(
                            "Definition work done progress token",
                        ))),
                    },
                    partial_result_params: PartialResultParams {
                        partial_result_token: Some(ProgressToken::String(String::from(
                            "Definition partial result progress token",
                        ))),
                    },
                }),
            )
            .await
            .unwrap()
            .await;
        dbg!(&response);
    }

    println!("TextDocumentFormatting");
    let now = Instant::now();
    let response = client
        .request::<TextDocumentRangeFormattingResult, (), _>(
            TextDocumentRangeFormatting::METHOD.into(),
            Some(DocumentRangeFormattingParams {
                work_done_progress_params: WorkDoneProgressParams {
                    work_done_token: Some(ProgressToken::String(String::from(
                        "Document formatting progress token",
                    ))),
                },
                text_document: TextDocumentIdentifier {
                    uri: String::from("file:///Users/romain/perso/virus/crates/virus/src/main.rs"),
                },
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 10,
                        character: 0,
                    },
                },
                options: FormattingOptions {
                    tab_size: 4,
                    insert_spaces: true,
                    trim_trailing_whitespace: Some(true),
                    insert_final_newline: Some(true),
                    trim_final_newlines: Some(true),
                },
            }),
        )
        .await
        .unwrap()
        .await;
    dbg!((&response, now.elapsed().as_millis()));

    println!("Shutdown");
    let response = client
        .request::<Null, (), ()>(Shutdown::METHOD.into(), None)
        .await
        .unwrap()
        .await;
    dbg!(&response);

    println!("Exit");
    client
        .notification::<()>(Exit::METHOD.into(), None)
        .await
        .unwrap();

    println!("Drop client");
    drop(client);
    println!("Await handle");
    handle.await.unwrap();
    println!("Kill child");
    child.kill().await.unwrap();
    println!("👋 Bye, have a good time!");
}

async fn wait(secs: u64) {
    for i in 0..secs {
        println!("Sleeping {}/10", i + 1);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
