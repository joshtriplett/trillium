use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::json;
use trillium::{async_trait, Conn, Handler, Status};

use crate::ApiConnExt;

/// A serde-serializable error
#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Error {
    /// An error occurred in parsing the provided body content
    #[error("Parse error at {path}: {message}")]
    ParseError {
        /// the path of the parse error, as provided by [`serde_path_to_error`]
        path: String,
        /// the contents of the error
        message: String,
    },
    /// A transmission error occurred in the connection to the http
    /// client
    #[error("I/O error type {kind}: {message}")]
    IoError {
        /// stringified [`std::io::ErrorKind`]
        kind: String,
        /// stringified [`std::io::Error`]
        message: String,
    },
    /// The client provided a content type that this library does not
    /// yet support
    #[error("Unsupported mime type: {mime_type}")]
    UnsupportedMimeType {
        /// the unsupported mime type
        mime_type: String,
    },
    /// Miscellaneous other errors -- please open an issue on
    /// trillium-api if you find yourself parsing the contents of
    /// this.
    #[error("{message}")]
    Other {
        /// A stringified error
        message: String,
    },
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::ParseError {
            path: format!("{}:{}", value.line(), value.column()),
            message: value.to_string(),
        }
    }
}

impl From<trillium_http::Error> for Error {
    fn from(error: trillium_http::Error) -> Self {
        match error {
            trillium_http::Error::Io(e) => Self::IoError {
                kind: e.kind().to_string(),
                message: e.to_string(),
            },

            other => Self::Other {
                message: other.to_string(),
            },
        }
    }
}

impl<E: Display> From<serde_path_to_error::Error<E>> for Error {
    fn from(e: serde_path_to_error::Error<E>) -> Self {
        Error::ParseError {
            path: e.path().to_string(),
            message: e.to_string(),
        }
    }
}

#[async_trait]
impl Handler for Error {
    async fn run(&self, conn: Conn) -> Conn {
        conn.with_json(&json!({ "error": self })).with_status(self)
    }
}

impl From<&Error> for Status {
    fn from(value: &Error) -> Self {
        match value {
            Error::ParseError { .. } => Status::UnprocessableEntity,
            Error::UnsupportedMimeType { .. } => Status::NotAcceptable,
            _ => Status::InternalServerError,
        }
    }
}
