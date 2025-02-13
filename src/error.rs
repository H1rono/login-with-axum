use std::fmt;

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectKind {
    Unauthorized,
    BadRequest,
    NotFound,
    Conflict,
}

impl fmt::Display for RejectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Unauthorized => "Unauthorized",
            Self::BadRequest => "Bad request",
            Self::NotFound => "Not found",
            Self::Conflict => "Conflict",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Reject {
    kind: RejectKind,
    message: String,
}

impl fmt::Display for Reject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl Reject {
    pub fn kind(&self) -> RejectKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error("{0}")]
    Reject(Reject),
    #[error(transparent)]
    Error(anyhow::Error),
}

impl From<Reject> for Failure {
    fn from(value: Reject) -> Self {
        Self::Reject(value)
    }
}

impl From<anyhow::Error> for Failure {
    fn from(value: anyhow::Error) -> Self {
        Failure::Error(value)
    }
}

impl Failure {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Reject {
            kind: RejectKind::Unauthorized,
            message: message.into(),
        }
        .into()
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Reject {
            kind: RejectKind::BadRequest,
            message: message.into(),
        }
        .into()
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Reject {
            kind: RejectKind::NotFound,
            message: message.into(),
        }
        .into()
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Reject {
            kind: RejectKind::Conflict,
            message: message.into(),
        }
        .into()
    }
}
