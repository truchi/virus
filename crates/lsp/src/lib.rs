pub mod client;
pub mod types {
    pub use super::generated::*;

    use serde::{Deserialize, Serialize};

    pub type Integer = i32;
    pub type UInteger = u32;
    pub type Decimal = f32;
    pub type Uri = String;
    pub type DocumentUri = Uri;

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
}
mod generated {
    pub mod client;
    pub mod enumerations;
    pub mod notifications;
    pub mod requests;
    pub mod server;
    pub mod structures;
    pub mod type_aliases;

    use super::{
        client::{LspClientNotify, LspClientRequest, LspClientRespond},
        transport::*,
        types::*,
    };
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json::Value;
    use std::{borrow::Cow, collections::HashMap};
    use tokio::io::AsyncWrite;
}
mod transport {
    mod lsp;
    mod rpc;

    pub use rpc::*;
}
