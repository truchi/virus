use lsp::{
    enumerations::PositionEncodingKind,
    structures::{
        ClientCapabilities, DefinitionParams, DocumentRangeFormattingParams, FormattingOptions,
        GeneralClientCapabilities, InitializeParams, InitializeParamsProcessId,
        InitializeParamsWorkspaceFolders, InitializedParams, PartialResultParams, Position, Range,
        TextDocumentIdentifier, TextDocumentPositionParams, WindowClientCapabilities,
        WorkDoneProgressParams, WorkspaceFolder,
    },
    type_aliases::{LspAny, ProgressToken},
    LspClient, ServerMessage,
};
use std::process::Stdio;
use tokio::{io::BufReader, process::Command};

#[tokio::main]
async fn main() {
    let mut child = Command::new("rust-analyzer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let (stdin, stdout) = (child.stdin.take().unwrap(), child.stdout.take().unwrap());
    let mut client = LspClient::new(BufReader::new(stdout), stdin, |message| match message {
        Ok(ServerMessage::ServerNotification(notification)) => {
            dbg!(notification);
        }
        Ok(ServerMessage::ServerRequest(request)) => {
            dbg!(request);
        }
        Err(error) => panic!("{error}"),
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
        .request()
        .initialize(InitializeParams {
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
            workspace_folders: Some(InitializeParamsWorkspaceFolders::WorkspaceFolderList(vec![
                WorkspaceFolder {
                    uri: String::from("file:///Users/romain/perso/virus"),
                    name: String::from("virus"),
                },
            ])),
        })
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    dbg!(response);

    println!("Initialized");
    client
        .notification()
        .initialized(InitializedParams {})
        .await
        .unwrap();

    wait(10).await;

    for _ in 0..0 {
        println!("TextDocumentDefinition");
        let response = client
            .request()
            .text_document_definition(DefinitionParams {
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
            })
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();
        dbg!(response);
    }

    println!("TextDocumentFormatting");
    let response = client
        .request()
        .text_document_range_formatting(DocumentRangeFormattingParams {
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
                tab_size: 2,
                insert_spaces: true,
                trim_trailing_whitespace: Some(true),
                insert_final_newline: Some(true),
                trim_final_newlines: Some(true),
            },
        })
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    dbg!(response);

    wait(3).await;

    println!("Shutdown");
    let response = client
        .request()
        .shutdown()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    dbg!(response);

    wait(3).await;

    println!("Exit");
    client.notification().exit().await.unwrap();

    println!("Drop client");
    drop(client);
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
