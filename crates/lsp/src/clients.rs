use crate::{
    structures::{InitializeParams, InitializedParams},
    LspClient, ServerMessage,
};
use std::{io, process::Stdio, sync::Arc};
use tokio::{
    io::BufReader,
    process::{ChildStdin, Command},
    sync::{Mutex, MutexGuard},
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LspClients                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type Handler = Box<dyn FnMut(io::Result<ServerMessage>) + Send>;

// TODO handle dead children
enum LspClientState {
    None {
        command: Option<Command>,
        // params: Option<InitializeParams>,
        handler: Option<Handler>,
    },
    Initialized {
        client: Arc<Mutex<LspClient<ChildStdin>>>,
    },
}

pub struct LspClients {
    rust: LspClientState,
}

impl LspClients {
    pub fn new(
        (
            rust_command,
            // rust_params,
            rust_handler,
        ): (
            Command,
            // InitializeParams,
            Handler,
        ),
    ) -> Self {
        Self {
            rust: LspClientState::None {
                command: Some(rust_command),
                // params: Some(rust_params),
                handler: Some(rust_handler),
            },
        }
    }

    // TODO because of workspace folders will aslo take ages, maybe we need to pass params here?
    pub fn rust(&mut self) -> Arc<Mutex<LspClient<ChildStdin>>> {
        match &mut self.rust {
            LspClientState::None {
                command,
                // params,
                handler,
            } => {
                let (
                    mut command,
                    // params,
                    handler,
                ) = (
                    command.take().unwrap(),
                    // params.take().unwrap(),
                    handler.take().unwrap(),
                );
                let mut child = command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap();
                let (stdin, stdout) = (
                    child.stdin.take().expect("Child stdin"),
                    child.stdout.take().expect("Child stdout"),
                );
                let client = LspClient::new(BufReader::new(stdout), stdin, handler);

                // client
                //     .request()
                //     .initialize(params)
                //     .await?
                //     .await?
                //     .expect("Initialize response");
                // client
                //     .notification()
                //     .initialized(InitializedParams {})
                //     .await?;

                // // TODO Wait for progress notifications
                // tokio::time::sleep(std::time::Duration::from_secs(10)).await;

                self.rust = LspClientState::Initialized {
                    client: Arc::new(Mutex::new(client)),
                };
            }
            LspClientState::Initialized { .. } => {}
        };

        match &self.rust {
            LspClientState::Initialized { client, .. } => client.clone(),
            _ => unreachable!(),
        }
    }
}
