//! ↩️ Initialize Request.
//!
//! The initialize request is sent as the first request from the client to the server.
//! If the server receives a request or notification before the initialize request
//! it should act as follows:
//! - For a request the response should be an error with code: -32002.
//!   The message can be picked by the server.
//! - Notifications should be dropped, except for the exit notification.
//!   This will allow the exit of a server without an initialize request.
//!
//! Until the server has responded to the initialize request with an InitializeResult,
//! the client must not send any additional requests or notifications to the server.
//! In addition the server is not allowed to send any requests or notifications to the client
//! until it has responded with an InitializeResult,
//! with the exception that during the initialize request the server is allowed to send
//! the notifications window/showMessage, window/logMessage and telemetry/event
//! as well as the window/showMessageRequest request to the client.
//! In case the client sets up a progress token in the initialize params
//! (e.g. property workDoneToken) the server is also allowed to use
//! that token (and only that token) using the $/progress notification sent
//! from the server to the client.
//!
//! The initialize request may only be sent once.

use crate::types::{Any, Integer, ProgressToken, TraceValue};
use serde::{Deserialize, Serialize};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Method                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub const METHOD: &'static str = "initialize";

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Params                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type Params = InitializeParams;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// An optional token that a server can use to report work done progress.
    pub work_done_token: ProgressToken,

    /// The process Id of the parent process that started the server. Is null if
    /// the process has not been started by another process. If the parent
    /// process is not alive then the server should exit (see exit notification)
    /// its process.
    #[serde(default)]
    pub process_id: Option<Integer>,

    /// Information about the client.
    #[serde(default)]
    pub client_info: Option<ClientInfo>,

    /// The locale the client is currently showing the user interface
    /// in. This must not necessarily be the locale of the operating
    /// system.
    ///
    /// Uses IETF language tags as the value's syntax
    /// (See https://en.wikipedia.org/wiki/IETF_language_tag)
    #[serde(default)]
    pub locale: Option<String>,

    /// User provided initialization options.
    #[serde(default)]
    pub initialization_options: Option<Any>,

    /// The capabilities provided by the client (editor or tool).
    pub capabilities: ClientCapabilities,

    /// The initial trace setting. If omitted trace is disabled ('off').
    #[serde(default)]
    pub trace: Option<TraceValue>,
    //
    // TODO
    // /// The workspace folders configured in the client when the server starts.
    // /// This property is only available if the client supports workspace folders.
    // /// It can be `null` if the client supports workspace folders but none are
    // /// configured.
    // #[serde(default)]
    // pub workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    /// The name of the client as defined by the client.
    pub name: String,

    /// The client's version as defined by the client.
    #[serde(default)]
    pub version: Option<String>,
}

// TODO
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Result                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type Result = InitializeResult;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// The capabilities the language server provides.
    pub capabilities: ServerCapabilities,

    /// Information about the server.
    #[serde(default)]
    pub server_info: Option<ServerInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// The name of the server as defined by the server.
    pub name: String,

    /// The server's version as defined by the server.
    #[serde(default)]
    pub version: Option<String>,
}

// TODO
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {}
