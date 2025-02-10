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

#[derive(Debug, Clone, thiserror::Error)]
pub enum Failure<E: Send + Sync + 'static = anyhow::Error> {
    #[error("{0}")]
    Reject(Reject),
    #[error(transparent)]
    Error(E),
}

impl<E> From<Reject> for Failure<E>
where
    E: Send + Sync + 'static,
{
    fn from(value: Reject) -> Self {
        Self::Reject(value)
    }
}

impl From<anyhow::Error> for Failure<anyhow::Error> {
    fn from(value: anyhow::Error) -> Self {
        Failure::Error(value)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>>
    for Failure<Box<dyn std::error::Error + Send + Sync + 'static>>
{
    fn from(value: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Failure::Error(value)
    }
}

impl<E: Send + Sync + 'static> Failure<E> {
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

impl<E1: Send + Sync + 'static> Failure<E1> {
    pub fn map<F, E2>(self, f: F) -> Failure<E2>
    where
        F: FnOnce(E1) -> E2,
        E2: Send + Sync + 'static,
    {
        match self {
            Self::Reject(r) => Failure::Reject(r),
            Self::Error(e1) => Failure::Error(f(e1)),
        }
    }
}
