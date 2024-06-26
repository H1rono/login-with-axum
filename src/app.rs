use anyhow::{anyhow, Context};
use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::{header::SET_COOKIE, HeaderMap};
use axum::response::{Html, Redirect};
use axum::Router;
use axum_extra::{headers::Cookie, TypedHeader};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{repository, AppState, Repository};

impl AppState {
    pub fn new(repo: Repository) -> Self {
        Self { repository: repo }
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
    let id = Uuid::new_v4();
    let user = repository::User {
        id: id.into(),
        display_id: req.display_id,
        name: req.name,
    };
    app.repository.create_user(user.clone()).await?;
    app.repository
        .save_raw_password(user.id, &req.password)
        .await?;
    Ok(Redirect::to("/login.html"))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUserRequest {
    pub display_id: String,
    pub password: String,
}

pub async fn login(
    State(app): State<AppState>,
    Form(req): Form<LoginUserRequest>,
) -> crate::Result<(HeaderMap, Redirect)> {
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
    let cookie_value = app.repository.create_session_for_user(user).await?;
    let headers: HeaderMap = [(
        SET_COOKIE,
        format!("ax_session={cookie_value}; Path=/; HttpOnly")
            .parse()
            .with_context(|| "failed to set cookie to header value")?,
    )]
    .into_iter()
    .collect();
    Ok((headers, Redirect::to("/me")))
}

pub async fn me(
    State(app): State<AppState>,
    TypedHeader(cookie): TypedHeader<Cookie>,
) -> crate::Result<Html<String>> {
    let session_cookie = cookie
        .get("ax_session")
        .ok_or_else(|| anyhow!("Unauthorized"))?;
    let user = app
        .repository
        .load_session_from_cookie(session_cookie)
        .await?
        .ok_or_else(|| anyhow!("user not found"))?;
    let repository::User {
        id,
        display_id,
        name,
    } = user;
    let html = std::fs::read_to_string("./public/me.html")
        .with_context(|| "failed to read public/me.html")?
        .replace("{{display_id}}", &display_id)
        .replace("{{name}}", &name)
        .replace("{{id}}", &id.to_string());
    Ok(Html(html))
}

pub fn public_routes() -> Router<AppState> {
    use tower_http::services::{Redirect, ServeFile};
    Router::new()
        .route_service(
            "/",
            Redirect::<Body>::permanent("/index.html".parse().unwrap()),
        )
        .route_service("/index.html", ServeFile::new("./public/index.html"))
        .route_service("/login.html", ServeFile::new("./public/login.html"))
        .route_service("/signup.html", ServeFile::new("./public/signup.html"))
}

pub fn api_routes() -> Router<AppState> {
    use axum::routing::post;
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}
