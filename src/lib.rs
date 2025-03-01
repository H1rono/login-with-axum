pub mod entity;
mod error;
pub mod provide;
mod registry;
mod repository;
mod router;
pub mod token;

use error::Failure;
pub use provide::State;
pub use registry::Registry;
pub use repository::Repository;
pub use router::make as make_router;

#[tracing::instrument]
pub async fn signal_handler() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!("{e}");
    }
}
