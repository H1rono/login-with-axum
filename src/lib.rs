use std::env;

use anyhow::Context;
use sqlx::mysql;

mod error;
mod model;
mod repository;
mod router;
mod token;

pub use error::{Error, Result};

#[must_use]
#[derive(Debug, Clone)]
pub struct Repository {
    pool: mysql::MySqlPool,
    bcrypt_cost: u32,
}

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

#[must_use]
#[derive(Clone)]
pub struct AppState {
    repository: Repository,
    token_manager: TokenManager,
    prefix: String,
}

pub fn make_router(state: AppState) -> axum::Router {
    let inner = axum::Router::new()
        .nest("/", router::public_routes(&state.prefix))
        .route("/ping", axum::routing::get(|| async { "pong" }))
        .route("/me", axum::routing::get(router::me))
        .nest("/api", router::api_routes());
    axum::Router::new()
        .nest(&state.prefix, inner)
        .with_state(state)
}

#[tracing::instrument]
pub async fn signal_handler() {
    if let Err(e) = tokio::signal::ctrl_c().await {
        tracing::error!("{e}");
    }
}
