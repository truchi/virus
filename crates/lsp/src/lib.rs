#![allow(unused)] // TODO: remove

// ↩️
// ↪️
// ➡️
// ⬅️

mod lifecycle {
    pub mod exit;
    pub mod initialize;
    pub mod initialized;
    pub mod log_trace;
    pub mod set_trace;
    pub mod shutdown;
}
mod protocol {
    pub mod lsp;
    pub mod rpc;
}
mod types;
