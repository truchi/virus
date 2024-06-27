use serde::{Deserialize, Serialize};

pub type Integer = i32;

pub type UInteger = u32;

pub type Decimal = f32;

pub type Any = serde_json::Value;

pub type Object = serde_json::Map<String, Any>;

pub type Array = Vec<Any>;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
#[serde(untagged)]
pub enum ProgressToken {
    Integer(Integer),
    String(String),
}

pub type Uri = url::Url;

pub type DocumentUri = Uri;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TraceValue {
    Off,
    Messages,
    Verbose,
}
