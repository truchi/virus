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
    sync::watch::{channel, Receiver},
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
//                                           LspClient                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type State = HashMap<Id, (Option<Response<Value, Value>>, Option<Waker>)>;

pub struct LspClient<W: AsyncWrite + Unpin> {
    id: Integer,
    writer: W,
    state: Arc<Mutex<State>>,
    handle: JoinHandle<()>,
    progress_receiver: Receiver<Option<WorkDone>>,
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
        let (progress_sender, progress_receiver) = channel(None);
        let state = Arc::new(Mutex::new(State::new()));
        let handle = tokio::spawn({
            let state = state.clone();

            async move {
                loop {
                    let message = match Message::<Value, Value>::read(&mut reader).await {
                        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => {
                            break;
                        }
                        message => message,
                    };

                    match message {
                        Ok(Message::Request(request)) => {
                            f(ServerRequest::deserialize(request)
                                .map(ServerMessage::ServerRequest));
                        }
                        Ok(Message::Notification(notification)) => {
                            f(ServerNotification::deserialize(notification)
                                .inspect(|notification| {
                                    match notification {
                                        ServerNotification::Progress(progress) => Some(progress),
                                        _ => None,
                                    }
                                    .and_then(|progress| WorkDone::try_from(progress).ok())
                                    .map(|work_done| progress_sender.send(Some(work_done)).ok());
                                })
                                .map(ServerMessage::ServerNotification));
                        }
                        Ok(Message::Response(response)) => {
                            let id = response.id.as_ref().expect("Response id");
                            let mut state = state.lock().expect("State lock");
                            let (response_slot, waker) = state.get_mut(id).expect("Request state");

                            debug_assert!(response_slot.is_none(), "Got two responses");
                            *response_slot = Some(response);

                            if let Some(waker) = waker.take() {
                                waker.wake();
                            }
                        }
                        Err(err) => f(Err(err)),
                    }
                }
            }
        });

        Self {
            id: 0,
            writer,
            state,
            handle,
            progress_receiver,
        }
    }

    pub async fn wait_for_work_done(&mut self) {
        const DELAY_MS: u64 = 1000;

        let mut tokens = HashSet::<Token>::new();

        while let Ok(()) = self.progress_receiver.changed().await {
            if let Some(work_done) = self.progress_receiver.borrow_and_update().as_ref() {
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

                match self.progress_receiver.has_changed() {
                    Ok(true) => continue,
                    _ => break,
                }
            }
        }
    }

    pub async fn test_wait(&mut self, secs: u64) -> impl Future<Output = ()> {
        println!("[LSP] Requesting (takes 200ms)");
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        println!("[LSP] Waiting for response ({secs}s)");
        futures::FutureExt::map(
            tokio::time::sleep(std::time::Duration::from_secs(secs)),
            move |_| println!("[LSP] Got response ({secs}s)"),
        )
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

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
enum Token {
    Integer(Integer),
    String(String),
}

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
