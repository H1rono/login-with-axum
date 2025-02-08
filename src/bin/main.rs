use anyhow::Context;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use login_with_axum as lib;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();
    let options = lib::conn_options_from_env("MYSQL_")
        .or_else(|_| lib::conn_options_from_env("MARIADB_"))
        .or_else(|_| lib::conn_options_from_env("NS_MARIADB_"))?;
    let repo = lib::Repository::connect_with(options).await?;
    repo.migrate().await?;
    let issuer = std::env::var("JWT_ISSUER").unwrap_or_else(|_| "login-with-axum".to_string());
    let jwt_key = std::env::var("JWT_KEY").context("JWT_KEY not found")?;
    let lifetime = std::env::var("JWT_LIFETIME")
        .unwrap_or_else(|_| "86400".to_string())
        .parse()
        .with_context(|| "failed to load JWT_LIFETIME as secs")?;
    let lifetime = std::time::Duration::from_secs(lifetime);
    let token_manager = lib::TokenManager::builder()
        .issuer(&issuer)
        .key(&jwt_key)
        .lifetime(lifetime)
        .build();
    let prefix = {
        let p = std::env::var("PREFIX").unwrap_or_default();
        let p = if p.starts_with('/') {
            p
        } else {
            format!("/{p}")
        };
        if p.ends_with('/') {
            p
        } else {
            format!("{p}/")
        }
    };
    let app_state = lib::AppState::new(repo, token_manager, &prefix);
    let app = lib::make_router(app_state).layer(TraceLayer::new_for_http());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "4176".to_string())
        .parse()?;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(lib::signal_handler())
        .await?;
    Ok(())
}
