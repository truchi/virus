mod client;
mod generated {
    pub mod client;
    pub mod enumerations;
    pub mod notifications;
    pub mod requests;
    pub mod server;
    pub mod structures;
    pub mod type_aliases;

    use super::{
        client::{LspClientNotification, LspClientRequest, LspClientResponse},
        transport::{Notification, Request, Response},
        *,
    };
    use serde::de::DeserializeOwned;
    use serde_json::Value;
    use std::collections::HashMap;
    use tokio::io::AsyncWrite;
}
mod transport {
    mod lsp;
    mod rpc;

    pub use rpc::*;
}

pub use client::*;
pub use generated::{enumerations, notifications, requests, server::*, structures, type_aliases};
pub use transport::{Code, Error, Id};

use serde::{Deserialize, Serialize};

/// Integer type.
pub type Integer = i32;
/// Unsigned integer type.
pub type UInteger = u32;
/// Decimal type.
pub type Decimal = f32;
/// URI type.
pub type Uri = String;
/// Document URI type.
pub type DocumentUri = Uri;

/// Null type.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Null;

impl Serialize for Null {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'de> Deserialize<'de> for Null {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Null;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("null")
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Null)
            }
        }

        deserializer.deserialize_unit(Visitor)
    }
}
