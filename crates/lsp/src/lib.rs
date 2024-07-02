#![allow(unused)] // TODO: remove

// ↩️
// ↪️
// ➡️
// ⬅️

pub mod client;
// mod lifecycle {
//     pub mod exit;
//     pub mod initialize;
//     pub mod initialized;
//     pub mod log_trace;
//     pub mod set_trace;
//     pub mod shutdown;
// }
pub mod generated {
    pub mod enumerations;
    pub mod notifications;
    pub mod requests;
    pub mod structures;
    pub mod type_aliases;

    use enumerations::*;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use std::collections::HashMap;
    use structures::*;
    use type_aliases::*;

    pub type Uri = String;
    pub type DocumentUri = Uri;
    pub type Null = (); // TODO
}
pub mod protocol {
    pub mod lsp;
    pub mod rpc;
}
mod types;
