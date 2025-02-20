use std::env;

use anyhow::Context;

pub mod entity;
mod error;
mod model;
mod repository;
mod router;
mod token;

pub use error::Failure;
pub use repository::Repository;
pub use router::{make as make_router, AppState};
pub use token::Manager as TokenManager;

pub fn conn_options_from_env(prefix: &str) -> anyhow::Result<repository::ConnectOptions> {
    let var = |suffix| {
        let var_name = format!("{prefix}{suffix}");
        env::var(&var_name).with_context(|| format!("failed to get env-var {var_name}"))
    };
    let hostname = var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
    let port: u16 = var("PORT")
        .unwrap_or_else(|_| "3306".to_string())
        .parse()
        .with_context(|| "failed to parse port number")?;
    let user = var("USER")?;
    let password = var("PASSWORD")?;
    let database = var("DATABASE")?;
    let options = repository::ConnectOptions::builder()
        .hostname(&hostname)
        .port(port)
        .username(&user)
        .password(&password)
        .database(&database)
        .build();
    Ok(options)
}

#[tracing::instrument]
pub async fn signal_handler() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!("{e}");
    }
}
