use crate::{
    transport::{Id, Message, Notification, Request, Response},
    types::Integer,
};
use futures::Future;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::{
    borrow::Cow,
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};
use tokio::{
    io::{AsyncBufRead, AsyncWrite},
    sync::mpsc::{channel, Receiver, Sender},
};

pub type RequestOrNotification = futures::future::Either<Request<Value>, Notification<Value>>;

type Requests = HashMap<Id, (Option<Response<Value, Value>>, Option<Waker>)>;

pub struct Client<W: AsyncWrite + Unpin> {
    id: Integer,
    writer: W,
    requests: Arc<Mutex<Requests>>,
    _sender: Sender<RequestOrNotification>,
}

impl<W: AsyncWrite + Unpin> Client<W> {
    pub fn new<R: 'static + AsyncBufRead + Send + Unpin>(
        mut reader: R,
        writer: W,
    ) -> (Self, Receiver<RequestOrNotification>) {
        let requests = Arc::new(Mutex::new(Requests::new()));
        let (sender, receiver) = channel(64);

        tokio::spawn({
            let requests = requests.clone();
            // Downgrade to allow the loop to return when self is dropped
            let sender = sender.downgrade();

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
                            if let Some(sender) = sender.upgrade() {
                                if sender
                                    .send(RequestOrNotification::Left(request))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        Message::Notification(notification) => {
                            if let Some(sender) = sender.upgrade() {
                                if sender
                                    .send(RequestOrNotification::Right(notification))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        Message::Response(response) => {
                            if let Some(id) = &response.id {
                                let mut requests = requests.lock().unwrap();
                                let (response_slot, waker) =
                                    requests.get_mut(id).expect("Request state");

                                debug_assert!(response_slot.is_none(), "Got two responses");
                                *response_slot = Some(response);

                                if let Some(waker) = waker.take() {
                                    waker.wake();
                                }
                            } else {
                                panic!("Got a response with no id");
                            }
                        }
                    }
                }
            }
        });

        (
            Self {
                id: 0,
                writer,
                requests,
                // We keep this here so the spawned task above finished when self is dropped
                _sender: sender,
            },
            receiver,
        )
    }

    pub async fn request<T: DeserializeOwned, E: DeserializeOwned, P: Serialize>(
        &mut self,
        method: Cow<'static, str>,
        params: Option<P>,
    ) -> io::Result<impl '_ + Future<Output = Response<T, E>>> {
        let id = Id::Integer({
            let id = self.id;
            self.id += 1;
            id
        });

        self.requests
            .lock()
            .unwrap()
            .insert(id.clone(), (None, None));

        Request::new(id.clone(), method, params)
            .write(&mut self.writer)
            .await?;

        Ok(futures::future::poll_fn(move |cx| {
            let mut requests = self.requests.lock().unwrap();
            let Some((response, waker)) = requests.get_mut(&id) else {
                // Was polled to completion already...
                return Poll::Pending;
            };

            if let Some(response) = response.take() {
                requests.remove(&id);

                Poll::Ready(response.deserialize().expect("Response deserialization"))
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

    pub async fn notification<P: Serialize>(
        &mut self,
        method: Cow<'static, str>,
        params: Option<P>,
    ) -> io::Result<()> {
        Notification::new(method, params)
            .write(&mut self.writer)
            .await
    }
}
