use crate::{
    generated::enumerations::{ErrorCodes, LspErrorCodes},
    transport::lsp,
    types::Integer,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{borrow::Cow, io};
use tokio::io::{AsyncBufRead, AsyncWrite};

// ────────────────────────────────────────────────────────────────────────────────────────────── //

pub const VERSION: &'static str = "2.0";

async fn read<T: DeserializeOwned, R: AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<T> {
    Ok(serde_json::from_slice(
        lsp::Message::read(reader).await?.content(),
    )?)
}

async fn write<T: Serialize, W: AsyncWrite + Unpin>(writer: &mut W, value: &T) -> io::Result<()> {
    lsp::Message::new(serde_json::to_vec(value)?)
        .write(writer)
        .await
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Id                                               //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Hash, Debug)]
#[serde(untagged)]
pub enum Id {
    Integer(Integer),
    String(Cow<'static, str>),
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Message                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Message<T, E> {
    Request(Request<T>),
    Notification(Notification<T>),
    Response(Response<T, E>),
}

impl<T, E> Message<T, E> {
    pub async fn read<R: AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<Self>
    where
        T: DeserializeOwned,
        E: DeserializeOwned,
    {
        read(reader).await
    }

    pub async fn write<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> io::Result<()>
    where
        T: Serialize,
        E: Serialize,
    {
        write(writer, self).await
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Request                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request<T> {
    /// `JSON-RPC` protocol version (must be exactly `"2.0"`).
    pub jsonrpc: Cow<'static, str>,

    /// The request id.
    pub id: Id,

    /// The method to be invoked.
    pub method: Cow<'static, str>,

    /// The method's parameters.
    #[serde(default = "Option::default")]
    pub params: Option<T>,
}

impl<T> Request<T> {
    pub fn new(id: Id, method: Cow<'static, str>, params: Option<T>) -> Self {
        Self {
            jsonrpc: VERSION.into(),
            id,
            method,
            params,
        }
    }

    pub async fn read<R: AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<Self>
    where
        T: DeserializeOwned,
    {
        read(reader).await
    }

    pub async fn write<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> io::Result<()>
    where
        T: Serialize,
    {
        write(writer, self).await
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                          Notification                                          //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Notification<T> {
    /// `JSON-RPC` protocol version (must be exactly `"2.0"`).
    pub jsonrpc: Cow<'static, str>,

    /// The method to be invoked.
    pub method: Cow<'static, str>,

    /// The method's parameters.
    #[serde(default = "Option::default")]
    pub params: Option<T>,
}

impl<T> Notification<T> {
    pub fn new(method: Cow<'static, str>, params: Option<T>) -> Self {
        Self {
            jsonrpc: VERSION.into(),
            method,
            params,
        }
    }

    pub async fn read<R: AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<Self>
    where
        T: DeserializeOwned,
    {
        read(reader).await
    }

    pub async fn write<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> io::Result<()>
    where
        T: Serialize,
    {
        write(writer, self).await
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Response                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Response<T, E> {
    /// `JSON-RPC` protocol version (must be exactly `"2.0"`).
    pub jsonrpc: Cow<'static, str>,

    /// The request id.
    #[serde(default = "Option::default")]
    pub id: Option<Id>,

    /// The result of a request.
    #[serde(default = "Option::default")]
    pub result: Option<T>,

    /// The error object in case a request fails.
    #[serde(default = "Option::default")]
    pub error: Option<Error<E>>,
}

impl<T, E> Response<T, E> {
    pub fn with_result(id: Option<Id>, result: T) -> Self {
        Self {
            jsonrpc: VERSION.into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn with_error(id: Option<Id>, error: Error<E>) -> Self {
        Self {
            jsonrpc: VERSION.into(),
            id,
            result: None,
            error: Some(error),
        }
    }

    pub async fn read<R: AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<Self>
    where
        T: DeserializeOwned,
        E: DeserializeOwned,
    {
        read(reader).await
    }

    pub async fn write<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> io::Result<()>
    where
        T: Serialize,
        E: Serialize,
    {
        write(writer, self).await
    }
}

impl Response<Value, Value> {
    pub fn deserialize<T: DeserializeOwned, E: DeserializeOwned>(
        self,
    ) -> io::Result<Response<T, E>> {
        debug_assert!(self.jsonrpc == VERSION);

        Ok(Response {
            jsonrpc: self.jsonrpc,
            id: self.id,
            result: self
                .result
                .map(|result| serde_json::from_value(result))
                .transpose()?,
            error: self
                .error
                .map(|error| {
                    Ok::<_, io::Error>(Error {
                        code: error.code,
                        message: error.message,
                        data: error
                            .data
                            .map(|data| serde_json::from_value(data))
                            .transpose()?,
                    })
                })
                .transpose()?,
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Error                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Error<T> {
    /// A number indicating the error type that occurred.
    pub code: Code,

    /// A string providing a short description of the error.
    pub message: Cow<'static, str>,

    /// A primitive or structured value that contains additional information about the error.
    #[serde(default = "Option::default")]
    pub data: Option<T>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Code                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
#[serde(from = "i32")]
#[serde(into = "i32")]
pub enum Code {
    //
    //
    // JSON-RPC
    //
    //
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    /// Error code indicating that a server received a notification or
    /// request before the server has received the `initialize` request.
    ServerNotInitialized,
    UnknownErrorCode,

    //
    //
    // LSP
    //
    //
    /// A request failed but it was syntactically correct, e.g the
    /// method name was known and the parameters were valid. The error
    /// message should contain human readable information about why
    /// the request failed.
    ///
    /// @since 3.17.0
    RequestFailed,
    /// The server cancelled the request. This error code should
    /// only be used for requests that explicitly support being
    /// server cancellable.
    ///
    /// @since 3.17.0
    ServerCancelled,
    /// The server detected that the content of a document got
    /// modified outside normal conditions. A server should
    /// NOT send this error code if it detects a content change
    /// in it unprocessed messages. The result even computed
    /// on an older state might still be useful for the client.
    ///
    /// If a client decides that a result is not of any use anymore
    /// the client should cancel the request.
    ContentModified,
    /// The client has canceled a request and a server has detected
    /// the cancel.
    RequestCancelled,

    //
    //
    // Custom
    //
    //
    /// Custom value.
    Custom(i32),
}

impl From<i32> for Code {
    fn from(value: i32) -> Self {
        match ErrorCodes::from(value) {
            ErrorCodes::ParseError => Self::ParseError,
            ErrorCodes::InvalidRequest => Self::InvalidRequest,
            ErrorCodes::MethodNotFound => Self::MethodNotFound,
            ErrorCodes::InvalidParams => Self::InvalidParams,
            ErrorCodes::InternalError => Self::InternalError,
            ErrorCodes::ServerNotInitialized => Self::ServerNotInitialized,
            ErrorCodes::UnknownErrorCode => Self::UnknownErrorCode,
            ErrorCodes::Custom(_) => match LspErrorCodes::from(value) {
                LspErrorCodes::RequestFailed => Self::RequestFailed,
                LspErrorCodes::ServerCancelled => Self::ServerCancelled,
                LspErrorCodes::ContentModified => Self::ContentModified,
                LspErrorCodes::RequestCancelled => Self::RequestCancelled,
                LspErrorCodes::Custom(_) => Self::Custom(value),
            },
        }
    }
}

impl Into<i32> for Code {
    fn into(self) -> i32 {
        match self {
            Code::ParseError => ErrorCodes::ParseError.into(),
            Code::InvalidRequest => ErrorCodes::InvalidRequest.into(),
            Code::MethodNotFound => ErrorCodes::MethodNotFound.into(),
            Code::InvalidParams => ErrorCodes::InvalidParams.into(),
            Code::InternalError => ErrorCodes::InternalError.into(),
            Code::ServerNotInitialized => ErrorCodes::ServerNotInitialized.into(),
            Code::UnknownErrorCode => ErrorCodes::UnknownErrorCode.into(),
            Code::RequestFailed => LspErrorCodes::RequestFailed.into(),
            Code::ServerCancelled => LspErrorCodes::ServerCancelled.into(),
            Code::ContentModified => LspErrorCodes::ContentModified.into(),
            Code::RequestCancelled => LspErrorCodes::RequestCancelled.into(),
            Code::Custom(value) => value,
        }
    }
}
