use crate::{LspClient, ServerMessage};
use std::{io, process::Stdio, sync::Arc};
use tokio::{
    io::BufReader,
    process::{ChildStdin, Command},
    sync::Mutex,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LspClients                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type Handler = Box<dyn FnMut(io::Result<ServerMessage>) + Send>;

// ────────────────────────────────────────────────────────────────────────────────────────────── //

// TODO handle dead children
enum LspClientState {
    None {
        command: Option<Command>,
        handler: Option<Handler>,
    },
    Initialized {
        client: Arc<Mutex<LspClient<ChildStdin>>>,
    },
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub struct LspClients {
    rust: LspClientState,
}

impl LspClients {
    pub fn new((rust_command, rust_handler): (Command, Handler)) -> Self {
        Self {
            rust: LspClientState::None {
                command: Some(rust_command),
                handler: Some(rust_handler),
            },
        }
    }

    pub fn rust(&mut self) -> Arc<Mutex<LspClient<ChildStdin>>> {
        match &mut self.rust {
            LspClientState::None { command, handler } => {
                let (mut command, handler) = (command.take().unwrap(), handler.take().unwrap());
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
                let client = Arc::new(Mutex::new(client));
                let clone = client.clone();

                self.rust = LspClientState::Initialized { client };

                clone
            }
            LspClientState::Initialized { client } => client.clone(),
        }
    }
}
