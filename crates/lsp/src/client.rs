use crate::{
    generated::server::{ServerNotification, ServerRequest, ServerResponse},
    transport::{Error, Id, Message, Notification, Request, Response},
    types::Integer,
};
use serde::Serialize;
use serde_json::Value;
use std::{
    borrow::Cow,
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncBufRead, AsyncWrite},
    task::JoinHandle,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         ServerMessage                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, PartialEq, Debug)]
pub enum ServerMessage {
    ServerNotification(ServerNotification),
    ServerRequest(ServerRequest),
    ServerResponse(ServerResponse),
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LspClient                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type State = Arc<Mutex<HashMap<Id, Cow<'static, str>>>>; // NOTE nice to have user data here?

pub struct LspClient<W: AsyncWrite + Unpin> {
    id: Integer,
    writer: W,
    state: State,
    handle: JoinHandle<()>,
}

impl<W: AsyncWrite + Unpin> LspClient<W> {
    pub fn new<
        R: 'static + AsyncBufRead + Send + Unpin,
        F: FnMut(io::Result<ServerMessage>) + Send + 'static,
    >(
        mut reader: R,
        writer: W,
        mut f: F,
    ) -> Self {
        let state = State::default();
        let handle = tokio::spawn({
            let state = state.clone();

            async move {
                loop {
                    let message = match Message::<Value, Value>::read(&mut reader).await {
                        Ok(message) => message,
                        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => {
                            break;
                        }
                        Err(err) => panic!("Cannot read message: {err:#?}"),
                    };

                    match message {
                        Message::Request(request) => {
                            f(ServerRequest::deserialize(request)
                                .map(ServerMessage::ServerRequest));
                        }
                        Message::Notification(notification) => {
                            f(ServerNotification::deserialize(notification)
                                .map(ServerMessage::ServerNotification));
                        }
                        Message::Response(response) => {
                            let id = response.id.as_ref().expect("Response id");
                            let mut state = state.lock().expect("State lock");
                            let method = state.remove(id).expect("Request state");

                            f(ServerResponse::deserialize(response, &method)
                                .map(ServerMessage::ServerResponse));
                        }
                    }
                }
            }
        });

        Self {
            id: 0,
            writer,
            state,
            handle,
        }
    }

    pub fn request(&mut self) -> LspClientRequest<W> {
        LspClientRequest { client: self }
    }

    pub fn notify(&mut self) -> LspClientNotify<W> {
        LspClientNotify { client: self }
    }

    pub fn respond(&mut self) -> LspClientRespond<W> {
        LspClientRespond { client: self }
    }
}

/// Private.
impl<W: AsyncWrite + Unpin> LspClient<W> {
    pub(crate) async fn send_notification<P: Serialize>(
        &mut self,
        method: Cow<'static, str>,
        params: Option<P>,
    ) -> io::Result<()> {
        Notification::new(method, params)
            .write(&mut self.writer)
            .await
    }

    pub(crate) async fn send_request<T: Serialize>(
        &mut self,
        method: Cow<'static, str>,
        params: Option<T>,
    ) -> io::Result<Id> {
        let id = Id::Integer({
            let id = self.id;
            self.id += 1;
            id
        });

        self.state
            .lock()
            .unwrap()
            .insert(id.clone(), method.clone());

        Request::new(id.clone(), method, params)
            .write(&mut self.writer)
            .await?;

        Ok(id)
    }

    pub(crate) async fn send_response<T: Serialize, E: Serialize>(
        &mut self,
        id: Option<Id>,
        data: Result<T, Error<E>>,
    ) -> io::Result<()> {
        match data {
            Ok(result) => Response::with_result(id, result),
            Err(error) => Response::with_error(id, error),
        }
        .write(&mut self.writer)
        .await
    }
}

impl<W: AsyncWrite + Unpin> Drop for LspClient<W> {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        LspClientRequest                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct LspClientRequest<'client, W: AsyncWrite + Unpin> {
    pub(crate) client: &'client mut LspClient<W>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        LspClientNotify                                         //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct LspClientNotify<'client, W: AsyncWrite + Unpin> {
    pub(crate) client: &'client mut LspClient<W>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        LspClientRespond                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct LspClientRespond<'client, W: AsyncWrite + Unpin> {
    pub(crate) client: &'client mut LspClient<W>,
}
