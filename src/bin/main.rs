use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use login_with_axum as lib;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();
    let options = lib::db_options_from_env("MYSQL_")
        .or_else(|_| lib::db_options_from_env("MARIADB_"))
        .or_else(|_| lib::db_options_from_env("NS_MARIADB_"))?;
    let repo = lib::Repository::connect_with(options).await?;
    repo.migrate().await?;
    let app_state = lib::AppState::new(repo);
    let app = lib::make_router(app_state).layer(TraceLayer::new_for_http());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "4176".to_string())
        .parse()?;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
