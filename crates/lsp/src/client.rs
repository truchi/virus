use crate::{
    notifications::NotificationTrait,
    requests::RequestTrait,
    transport::{Message, Notification, Request, Response},
    Error, Id, Integer, ServerNotification, ServerRequest,
};
use futures::Future;
use serde_json::Value;
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
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
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                           LspClient                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

type State = Arc<Mutex<HashMap<Id, (Option<Response<Value, Value>>, Option<Waker>)>>>;

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
