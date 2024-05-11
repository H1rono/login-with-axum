use std::env;

use anyhow::Context;
use async_sqlx_session::MySqlSessionStore;
use sqlx::mysql;

mod repository;

#[must_use]
#[derive(Debug, Clone)]
struct Repository {
    pool: mysql::MySqlPool,
    session_store: MySqlSessionStore,
    bcrypt_cost: u32,
}

pub fn db_options_from_env(prefix: &str) -> anyhow::Result<mysql::MySqlConnectOptions> {
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
    let options = mysql::MySqlConnectOptions::new()
        .host(&hostname)
        .port(port)
        .username(&user)
        .password(&password)
        .database(&database);
    Ok(options)
}

pub fn make_router() -> axum::Router {
    axum::Router::new().route("/ping", axum::routing::get(|| async { "pong" }))
}
