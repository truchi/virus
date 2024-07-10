use crate::{
    notifications::NotificationTrait,
    requests::RequestTrait,
    structures::ProgressParams,
    transport::{Message, Notification, Request, Response},
    type_aliases::{LspAny, ProgressToken},
    Error, Id, Integer, ServerNotification, ServerRequest,
};
use futures::Future;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    io,
    str::FromStr,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
    time::Duration,
};
use tokio::{
    io::{AsyncBufRead, AsyncWrite},
    process::ChildStdin,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        watch::{channel, Receiver},
    },
    task::JoinHandle,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                         ServerMessage                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Clone, PartialEq, Debug)]
pub enum ServerMessage {
    ServerNotification(ServerNotification),
    ServerRequest(ServerRequest),
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                      ServerMessageSender                                       //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type ServerMessageSender = UnboundedSender<ServerMessage>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                     ServerMessageReceiver                                      //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type ServerMessageReceiver = UnboundedReceiver<ServerMessage>;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LspClient                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type State = HashMap<Id, (Option<Response<Value, Value>>, Option<Waker>)>;

pub struct LspClient<W: AsyncWrite + Unpin = ChildStdin> {
    id: Integer,
    writer: W,
    state: Arc<Mutex<State>>,
    handle: JoinHandle<()>,
    work_done_receiver: Receiver<Option<WorkDone>>,
}

impl<W: AsyncWrite + Unpin> LspClient<W> {
    pub fn new<R: AsyncBufRead + Send + Unpin + 'static>(
        mut reader: R,
        writer: W,
        server_message_sender: ServerMessageSender,
    ) -> Self {
        let (work_done_sender, work_done_receiver) = channel(None);
        let state = Arc::new(Mutex::new(State::new()));
        let handle = tokio::spawn({
            let state = state.clone();

            async move {
                loop {
                    let message = match Message::<Value, Value>::read(&mut reader).await {
                        Ok(message) => message,
                        Err(err) => {
                            dbg!(err);
                            break;
                        }
                    };

                    match message {
                        Message::Request(request) => {
                            let request = ServerMessage::ServerRequest(
                                ServerRequest::deserialize(request)
                                    .expect("ServerRequest::deserialize"),
                            );

                            server_message_sender
                                .send(request)
                                .expect("Send server message");
                        }
                        Message::Notification(notification) => {
                            let notification = ServerNotification::deserialize(notification)
                                .expect("ServerNotification::deserialize");

                            match &notification {
                                ServerNotification::Progress(progress) => Some(progress),
                                _ => None,
                            }
                            .and_then(|progress| WorkDone::try_from(progress).ok())
                            .map(|work_done| work_done_sender.send(Some(work_done)).ok());

                            server_message_sender
                                .send(ServerMessage::ServerNotification(notification))
                                .expect("Send server message");
                        }
                        Message::Response(response) => {
                            let id = response.id.as_ref().expect("Response id");
                            let mut state = state.lock().expect("State lock");
                            let (response_slot, waker) = state.get_mut(id).expect("Request state");

                            debug_assert!(response_slot.is_none(), "Got two responses");
                            *response_slot = Some(response);

                            if let Some(waker) = waker.take() {
                                waker.wake();
                            }
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
            work_done_receiver,
        }
    }

    pub async fn wait_for_work_done(&mut self) {
        const DELAY_MS: u64 = 1000;

        let mut tokens = HashSet::<Token>::new();

        while let Ok(()) = self.work_done_receiver.changed().await {
            if let Some(work_done) = self.work_done_receiver.borrow_and_update().as_ref() {
                match work_done.kind {
                    Kind::Begin | Kind::Report => {
                        tokens.insert(work_done.token.clone());
                    }
                    Kind::End => {
                        tokens.remove(&work_done.token);
                    }
                }
            }

            if tokens.is_empty() {
                tokio::time::sleep(Duration::from_millis(DELAY_MS)).await;

                match self.work_done_receiver.has_changed() {
                    Ok(true) => continue,
                    _ => break,
                }
            }
        }
    }

    pub fn notification(&mut self) -> LspClientNotification<W> {
        LspClientNotification { client: self }
    }

    pub fn request(&mut self) -> LspClientRequest<W> {
        LspClientRequest { client: self }
    }

    pub fn response(&mut self) -> LspClientResponse<W> {
        LspClientResponse { client: self }
    }
}

/// Private.
impl<W: AsyncWrite + Unpin> LspClient<W> {
    pub(crate) async fn send_notification<T: NotificationTrait>(
        &mut self,
        params: T::Params,
    ) -> io::Result<()> {
        Notification::new(T::METHOD.into(), T::params(params))
            .write(&mut self.writer)
            .await
    }

    pub(crate) async fn send_request<T: RequestTrait>(
        &mut self,
        params: T::Params,
    ) -> io::Result<impl '_ + Future<Output = io::Result<Result<T::Result, Error<T::Error>>>>> {
        let id = Id::Integer({
            let id = self.id;
            self.id += 1;
            id
        });

        self.state.lock().unwrap().insert(id.clone(), (None, None));

        Request::new(id.clone(), T::METHOD.into(), T::params(params))
            .write(&mut self.writer)
            .await?;

        Ok(futures::future::poll_fn(move |cx| {
            let mut state = self.state.lock().unwrap();
            let Some((response, waker)) = state.get_mut(&id) else {
                // Was polled to completion already...
                return Poll::Pending;
            };

            if let Some(response) = response.take() {
                state.remove(&id);

                Poll::Ready(T::deserialize_response(response))
            } else {
                if let Some(waker) = waker {
                    waker.clone_from(cx.waker());
                } else {
                    *waker = Some(cx.waker().clone());
                }

                Poll::Pending
            }
        }))
    }

    pub(crate) async fn send_response<T: RequestTrait>(
        &mut self,
        id: Option<Id>,
        data: Result<T::Result, Error<T::Error>>,
    ) -> io::Result<()> {
        match data {
            Ok(result) => Response::with_result(id, result),
            Err(error) => Response::with_error(id, T::error(error)),
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
//                                     LspClientNotification                                      //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct LspClientNotification<'client, W: AsyncWrite + Unpin> {
    pub(crate) client: &'client mut LspClient<W>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        LspClientRequest                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct LspClientRequest<'client, W: AsyncWrite + Unpin> {
    pub(crate) client: &'client mut LspClient<W>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       LspClientResponse                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub struct LspClientResponse<'client, W: AsyncWrite + Unpin> {
    pub(crate) client: &'client mut LspClient<W>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            WorkDone                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

// TODO It could be useful to have correctly deserialized progress backed in

struct WorkDone {
    token: Token,
    kind: Kind,
}

impl WorkDone {
    const KIND: &'static str = "kind";
}

impl TryFrom<&ProgressParams> for WorkDone {
    type Error = ();

    fn try_from(progress: &ProgressParams) -> Result<Self, Self::Error> {
        let token = match &progress.token {
            ProgressToken::Integer(integer) => Token::Integer(*integer),
            ProgressToken::String(string) => Token::String(string.clone()),
        };

        match &progress.value {
            LspAny::LspObject(value) => value.get(Self::KIND),
            _ => None,
        }
        .and_then(|kind| match kind {
            LspAny::String(kind) => Some(WorkDone {
                token,
                kind: kind.parse().ok()?,
            }),
            _ => None,
        })
        .ok_or(())
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum Token {
    Integer(Integer),
    String(String),
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

#[derive(PartialEq)]
enum Kind {
    Begin,
    Report,
    End,
}

impl Kind {
    const BEGIN: &'static str = "begin";
    const REPORT: &'static str = "report";
    const END: &'static str = "end";
}

impl FromStr for Kind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::BEGIN => Ok(Self::Begin),
            Self::REPORT => Ok(Self::Report),
            Self::END => Ok(Self::End),
            _ => Err(()),
        }
    }
}
