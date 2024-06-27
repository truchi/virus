//! ⬅️ LogTrace Notification.
//!
//! A notification to log the trace of the server’s execution.
//! The amount and content of these notifications depends on the current trace configuration.
//! If trace is 'off', the server should not send any logTrace notification.
//! If trace is 'messages', the server should not add the 'verbose' field in the LogTraceParams.
//!
//! $/logTrace should be used for systematic trace reporting.
//! For single debugging messages, the server should send window/logMessage notifications.

use crate::types::TraceValue;
use serde::{Deserialize, Serialize};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Method                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub const METHOD: &'static str = "$/logTrace";

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                             Params                                             //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

pub type Params = LogTraceParams;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LogTraceParams {
    /// The message to be logged.
    message: String,

    /// Additional information that can be computed if the `trace` configuration
    /// is set to `'verbose'`.
    verbose: Option<String>,
}
