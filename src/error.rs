use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[must_use]
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] anyhow::Error);

impl IntoResponse for Error {
    #[tracing::instrument(skip_all)]
    fn into_response(self) -> Response {
        let error = self.0;
        tracing::error!("{error:?}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub type Result<T, E = Error> = ::std::result::Result<T, E>;
