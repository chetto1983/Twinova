//! The channel's messages, one JSON object per line.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A connection's first line: the launch's token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hello {
    pub token: String,
}

/// A command for the app: a method of its registry, or one the channel answers itself (`screenshot`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// The answer to request `id`, or to the hello (id 0): its result, or why it failed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reply {
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Reply {
    pub fn of(id: u64, answer: Result<Value, String>) -> Self {
        match answer {
            Ok(result) => Self {
                id,
                result: Some(result),
                error: None,
            },
            Err(error) => Self {
                id,
                result: None,
                error: Some(error),
            },
        }
    }

    pub fn answer(self) -> Result<Value, String> {
        match (self.result, self.error) {
            (_, Some(error)) => Err(error),
            (result, None) => Ok(result.unwrap_or(Value::Null)),
        }
    }
}
