use anyhow::{anyhow, Context};
use axum::extract::{Form, State};
use axum::response::Redirect;
use axum::{Json, Router};
use axum_extra::extract::cookie;
use serde::{Deserialize, Serialize};

use crate::model::User;
use crate::{Repository, TokenManager};

#[must_use]
#[derive(Clone)]
pub struct AppState {
    repository: Repository,
    token_manager: TokenManager,
    prefix: String,
}

impl AppState {
    pub fn new(repo: Repository, tm: TokenManager, prefix: &str) -> Self {
        Self {
            repository: repo,
            token_manager: tm,
            prefix: prefix.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterUserRequest {
    pub display_id: String,
    pub name: String,
    pub password: String,
}

pub async fn register(
    State(app): State<AppState>,
    Form(req): Form<RegisterUserRequest>,
) -> crate::Result<Redirect> {
    // TODO: validation
    let id = uuid::Uuid::new_v4();
    let user = User {
        id: id.into(),
        display_id: req.display_id,
        name: req.name,
    };
    app.repository.create_user(user.clone()).await?;
    app.repository
        .save_raw_password(user.id, &req.password)
        .await?;
    Ok(Redirect::to(&format!("{}login.html", &app.prefix)))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUserRequest {
    pub display_id: String,
    pub password: String,
}

pub async fn login(
    State(app): State<AppState>,
    cookie_jar: cookie::CookieJar,
    Form(req): Form<LoginUserRequest>,
) -> crate::Result<(cookie::CookieJar, Redirect)> {
    let user = app
        .repository
        .get_user_by_display_id(&req.display_id)
        .await
        .with_context(|| "failed to get user by display id")?
        .ok_or_else(|| anyhow!("user not found"))?;
    let verification = app
        .repository
        .verify_user_password(user.id, &req.password)
        .await
        .with_context(|| "failed to verify password")?
        .ok_or_else(|| anyhow!("password not found"))?;
    if !verification {
        return Err(anyhow!("Unauthorized").into());
    }
    let cookie_value = app
        .token_manager
        .encode(user.id)
        .with_context(|| "encoding to JWT failed")?;
    let cookie = cookie::Cookie::build(("ax_session", cookie_value))
        .path(app.prefix.clone())
        .http_only(true)
        .build();
    let cookie_jar = cookie_jar.add(cookie);
    Ok((cookie_jar, Redirect::to(&format!("{}me.html", app.prefix))))
}

pub async fn logout(
    State(app): State<AppState>,
    cookie_jar: cookie::CookieJar,
) -> crate::Result<(cookie::CookieJar, Redirect)> {
    let _cookie = cookie_jar
        .get("ax_session")
        .ok_or_else(|| anyhow!("Unauthorized"))?;
    // TODO Expire within TokenManager
    // TODO: add attribute `Expires` with chrono
    let cookie = cookie::Cookie::build("ax_session")
        .removal()
        .path(app.prefix.clone())
        .http_only(true)
        .build();
    let cookie_jar = cookie_jar.add(cookie);
    Ok((cookie_jar, Redirect::to(&app.prefix)))
}

pub async fn me(
    State(app): State<AppState>,
    cookie_jar: cookie::CookieJar,
) -> crate::Result<Json<User>> {
    let session_cookie = cookie_jar
        .get("ax_session")
        .context("Unauthenticated")?
        .value();
    let user_id = app
        .token_manager
        .decode(session_cookie)
        .with_context(|| "failed to parse cookie value")?;
    let user = app
        .repository
        .get_user_by_id(user_id)
        .await
        .with_context(|| "failed to get user by id")?
        .with_context(|| "user not found")?;
    Ok(Json(user))
}

pub fn api_routes() -> Router<AppState> {
    use axum::routing::{get, post};
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

pub fn make(state: AppState) -> axum::Router {
    use tower_http::services::ServeDir;
    let inner = axum::Router::new()
        .route("/ping", axum::routing::get(|| async { "pong" }))
        .nest("/api", api_routes())
        .fallback_service(ServeDir::new("./public"));
    let router = if &state.prefix == "/" {
        inner
    } else {
        axum::Router::new().nest(&state.prefix, inner)
    };
    router.with_state(state)
}
