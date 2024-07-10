use crate::{LspClient, ServerMessageSender};
use std::{process::Stdio, sync::Arc};
use tokio::{io::BufReader, process::Command, sync::Mutex};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LspClients                                           //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO handle dead children
enum State {
    None {
        command: Option<Command>,
        server_message_sender: Option<ServerMessageSender>,
    },
    Initialized {
        client: Arc<Mutex<LspClient>>,
    },
}

pub struct LspClients {
    rust: State,
}

impl LspClients {
    pub fn new((rust_command, rust_server_message_sender): (Command, ServerMessageSender)) -> Self {
        Self {
            rust: State::None {
                command: Some(rust_command),
                server_message_sender: Some(rust_server_message_sender),
            },
        }
    }

    pub fn rust(&mut self) -> Arc<Mutex<LspClient>> {
        match &mut self.rust {
            State::None {
                command,
                server_message_sender,
            } => {
                let (mut command, server_message_sender) = (
                    command.take().unwrap(),
                    server_message_sender.take().unwrap(),
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
                let client = LspClient::new(BufReader::new(stdout), stdin, server_message_sender);
                let client = Arc::new(Mutex::new(client));
                let clone = client.clone();

                self.rust = State::Initialized { client };

                clone
            }
            State::Initialized { client } => client.clone(),
        }
    }
}
