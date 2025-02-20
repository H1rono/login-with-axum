use login_with_axum as lib;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    use std::sync::Arc;

    use futures::TryFutureExt;
    use tower_http::trace::TraceLayer;
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();

    let pool = load::pool("MYSQL_")
        .or_else(|_| load::pool("MARIADB_"))
        .or_else(|_| load::pool("NS_MARIADB_"))
        .await?;
    let jwt_config = load::jwt_config()?;
    let bcrypt_cost = load::bcrypt_cost()?;
    let path_prefix = load::path_prefix();
    let cookie_name = load::cookie_name();
    let state = lib::State {
        token_config: jwt_config,
        bcrypt_cost,
        path_prefix,
        cookie_name,
        pool,
        impls: lib::provide::Impls::default(),
    };
    state.setup().await?;
    let app = lib::make_router(Arc::new(state)).layer(TraceLayer::new_for_http());
    let port: u16 = load::port()?;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "Listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(lib::signal_handler())
        .await?;
    Ok(())
}

mod load {
    use anyhow::Context;

    use login_with_axum as lib;

    #[tracing::instrument]
    pub async fn pool(env_prefix: &str) -> anyhow::Result<sqlx::MySqlPool> {
        let var = |suffix| {
            let var_name = format!("{env_prefix}{suffix}");
            std::env::var(&var_name).with_context(|| format!("Failed to get env-var {var_name}"))
        };
        let hostname = var("HOSTNAME")?;
        let port: u16 = var("PORT")?
            .parse()
            .with_context(|| "Failed to parse port number")?;
        let user = var("USER")?;
        let password = var("PASSWORD")?;
        let database = var("DATABASE")?;
        let options = sqlx::mysql::MySqlConnectOptions::new()
            .host(&hostname)
            .port(port)
            .username(&user)
            .password(&password)
            .database(&database);
        let pool = sqlx::MySqlPool::connect_with(options)
            .await
            .context("Failed to connect database")
            .inspect_err(|e| tracing::error!("{e:?}"))?;
        Ok(pool)
    }

    pub fn jwt_config() -> anyhow::Result<lib::token::JwtConfigImpl> {
        let issuer = std::env::var("JWT_ISSUER").unwrap_or_else(|_| "login-with-axum".to_string());
        let key = std::env::var("JWT_KEY").context("JWT_KEY not found")?;
        let lifetime = std::env::var("JWT_LIFETIME")
            .unwrap_or_else(|_| "86400".to_string())
            .parse()
            .with_context(|| "Failed to load JWT_LIFETIME as secs")?;
        let lifetime = std::time::Duration::from_secs(lifetime);
        let config = lib::token::Jwt::config_builder()
            .issuer(&issuer)
            .key(&key)
            .lifetime(lifetime)
            .build();
        Ok(config)
    }

    pub fn bcrypt_cost() -> anyhow::Result<u32> {
        let cost = std::env::var("BCRYPT_COST")
            .unwrap_or_else(|_| bcrypt::DEFAULT_COST.to_string())
            .parse()
            .context("Failed to parse BCRYPT_COST as u32")?;
        Ok(cost)
    }

    pub fn path_prefix() -> String {
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
    }

    pub fn cookie_name() -> String {
        std::env::var("COOKIE_NAME").unwrap_or_else(|_| "ax_session".to_string())
    }

    pub fn port() -> anyhow::Result<u16> {
        std::env::var("PORT")
            .unwrap_or_else(|_| 4176.to_string())
            .parse()
            .context("Failed to parse PORT value as u16")
    }
}
