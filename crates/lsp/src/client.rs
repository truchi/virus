use crate::{
    protocol::{
        lsp::Message,
        rpc::{Id, Request, Response},
    },
    types::{Any, Integer},
};
use futures::Future;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashMap,
    process::Stdio,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};
use tokio::{
    io::{AsyncBufRead, AsyncReadExt, AsyncWrite},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::mpsc::Sender,
};

type Requests = HashMap<Id, (Option<Response<Any>>, Option<Waker>)>;

pub struct Client<W: AsyncWrite + Unpin> {
    id: Integer,
    writer: W,
    requests: Arc<Mutex<Requests>>,
}

impl<W: AsyncWrite + Unpin> Client<W> {
    pub fn new<R: 'static + AsyncBufRead + Send + Unpin>(
        mut reader: R,
        writer: W,
        notifications: Sender<Request<Any>>,
    ) -> std::io::Result<Self> {
        let requests = Arc::new(Mutex::new(Requests::new()));

        tokio::spawn({
            let mut requests = requests.clone();

            async move {
                loop {
                    let message = Message::read(&mut reader).await.expect("Read message");
                    let response =
                        serde_json::from_slice::<Response<crate::types::Any>>(&message.content)
                            .unwrap();

                    if let Some(id) = response.id.clone() {
                        let mut requests = requests.lock().unwrap();
                        let (response_slot, waker) = requests.get_mut(&id).expect("Request state");

                        debug_assert!(response_slot.is_none());
                        *response_slot = Some(response);

                        if let Some(waker) = waker.take() {
                            waker.wake();
                        }
                    } else {
                        // TODO: Send notification to caller
                        // notifications.send(response)
                    }
                }
            }
        });

        Ok(Self {
            id: 0,
            writer,
            requests,
        })
    }

    pub async fn request<T: Serialize, U: DeserializeOwned>(
        &mut self,
        method: String,
        params: Option<T>,
    ) -> std::io::Result<impl '_ + Future<Output = Response<U>>> {
        let id = Id::Integer(self.id());

        self.requests
            .lock()
            .unwrap()
            .insert(id.clone(), (None, None));

        let request = Request::request(id.clone(), method, params);
        let message = Message::new(serde_json::to_vec(&request).expect("Serialize request"));
        message
            .write(&mut self.writer)
            .await
            .expect("Write message");

        Ok(futures::future::poll_fn(move |cx| {
            let mut requests = self.requests.lock().unwrap();
            let Some((response, waker)) = requests.get_mut(&id) else {
                // Was polled to completion already...
                return Poll::Pending;
            };

            if let Some(response) = response.take() {
                requests.remove(&id);

                Poll::Ready(Response {
                    jsonrpc: response.jsonrpc,
                    id: response.id,
                    result: response
                        .result
                        .map(|result| serde_json::from_value(result).unwrap()),
                    error: response.error,
                })
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

    fn id(&mut self) -> Integer {
        let id = self.id;
        self.id += 1;
        id
    }
}

// ────────────────────────────────────────────────────────────────────────────────────────────── //

// pub struct Client {
//     child: Child,
//     stdin: ChildStdin,
// }

// impl Client {
//     pub fn new<'a>(mut command: Command) -> std::io::Result<Self> {
//         let mut child = command
//             .stdin(Stdio::piped())
//             .stdout(Stdio::piped())
//             .spawn()?;
//         let stdin = child.stdin.take().unwrap();
//         let mut stdout = child.stdout.take().unwrap();

//         let (tx, mut rx) = channel::<String>(100);

//         tokio::spawn(async move {
//             let mut buffer = vec![0; 1024];

//             loop {
//                 match stdout.read_exact(&mut buffer).await {
//                     Ok(_) => {
//                         // if tx.send(buffer.clone()).await.is_err() {
//                         //     unreachable!();
//                         // }
//                     }
//                     Err(_) => break,
//                 }
//             }
//         });

//         Ok(Self { child, stdin })
//     }
// }
