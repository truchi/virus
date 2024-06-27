use crate::types::{Any, Array, Integer, Object};
use serde::{Deserialize, Serialize};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                               Id                                               //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Id {
    Integer(Integer),
    String(String),
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Request                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request<T> {
    /// `JSON-RPC` protocol version (must be exactly `"2.0"`).
    pub jsonrpc: &'static str,

    /// The request id (`None` for notifications).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,

    /// The method to be invoked.
    pub method: String,

    /// The method's parameters.
    #[serde(default)]
    pub params: Option<T>,
}

impl<T> Request<T> {
    pub const VERSION: &'static str = "2.0";

    pub fn request(id: Id, method: String, params: Option<T>) -> Self {
        Self {
            jsonrpc: Self::VERSION,
            id: Some(id),
            method,
            params,
        }
    }

    pub fn notification(method: String, params: Option<T>) -> Self {
        Self {
            jsonrpc: Self::VERSION,
            id: None,
            method,
            params,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                            Response                                            //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Response<T> {
    /// `JSON-RPC` protocol version (must be exactly `"2.0"`).
    pub jsonrpc: &'static str,

    /// The request id.
    #[serde(default)]
    pub id: Option<Id>,

    /// The result of a request.
    #[serde(default)]
    pub result: Option<T>,

    /// The error object in case a request fails.
    #[serde(default)]
    pub error: Option<Error>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Error                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Error {
    /// A number indicating the error type that occurred.
    pub code: Code,

    /// A string providing a short description of the error.
    pub message: String,

    /// A primitive or structured value that contains additional information about the error.
    #[serde(default)]
    pub data: Option<Any>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                              Code                                              //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Code {
    /// (`RPC`) Invalid JSON was received by the server. An error occurred
    /// on the server while parsing the JSON text.
    ParseError = -32700,

    /// (`RPC`) The JSON sent is not a valid Request object.
    InvalidRequest = -32600,

    /// (`RPC`) The method does not exist / is not available.
    MethodNotFound = -32601,

    /// (`RPC`) Invalid method parameter(s).
    InvalidParams = -32602,

    /// (`RPC`) Internal JSON-RPC error.
    InternalError = -32603,

    /// (`LSP`) Error code indicating that a server received a notification or
    /// request before the server has received the `initialize` request.
    ServerNotInitialized = -32002,
    UnknownErrorCode = -32001,

    /// (`LSP`) A request failed but it was syntactically correct, e.g the
    /// method name was known and the parameters were valid. The error
    /// message should contain human readable information about why
    /// the request failed.
    RequestFailed = -32803,

    /// (`LSP`) The server cancelled the request. This error code should
    /// only be used for requests that explicitly support being
    /// server cancellable.
    ServerCancelled = -32802,

    /// (`LSP`) The server detected that the content of a document got
    /// modified outside normal conditions. A server should
    /// NOT send this error code if it detects a content change
    /// in it unprocessed messages. The result even computed
    /// on an older state might still be useful for the client.
    ///
    /// If a client decides that a result is not of any use anymore
    /// the client should cancel the request.
    ContentModified = -32801,

    /// (`LSP`) The client has canceled a request and a server has detected
    /// the cancel.
    RequestCancelled = -32800,
}

impl Serialize for Code {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as Integer)
    }
}

struct CodeVisitor;

impl<'de> serde::de::Visitor<'de> for CodeVisitor {
    type Value = Code;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid error code integer")
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(match value {
            _ if value == Code::ParseError as Integer => Code::ParseError,
            _ if value == Code::InvalidRequest as Integer => Code::InvalidRequest,
            _ if value == Code::MethodNotFound as Integer => Code::MethodNotFound,
            _ if value == Code::InvalidParams as Integer => Code::InvalidParams,
            _ if value == Code::InternalError as Integer => Code::InternalError,
            _ if value == Code::ServerNotInitialized as Integer => Code::ServerNotInitialized,
            _ if value == Code::UnknownErrorCode as Integer => Code::UnknownErrorCode,
            _ if value == Code::RequestFailed as Integer => Code::RequestFailed,
            _ if value == Code::ServerCancelled as Integer => Code::ServerCancelled,
            _ if value == Code::ContentModified as Integer => Code::ContentModified,
            _ if value == Code::RequestCancelled as Integer => Code::RequestCancelled,

            // Using convenient `UnknownErrorCode` when unknown
            _ => Code::UnknownErrorCode,
        })
    }
}

impl<'de> Deserialize<'de> for Code {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_i32(CodeVisitor)
    }
}
