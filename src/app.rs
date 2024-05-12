use anyhow::{anyhow, Context};
use axum::extract::{Json, State};
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
    Json(req): Json<RegisterUserRequest>,
) -> crate::Result<Json<repository::User>> {
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
    Ok(Json(user))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUserRequest {
    pub display_id: String,
    pub password: String,
}

pub async fn login(
    State(app): State<AppState>,
    Json(req): Json<LoginUserRequest>,
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
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        format!("cookie={cookie_value}")
            .parse()
            .with_context(|| "failed to set cookie to header value")?,
    );
    Ok((headers, Redirect::to("/me")))
}

pub async fn me(
    State(app): State<AppState>,
    TypedHeader(cookie): TypedHeader<Cookie>,
) -> crate::Result<Html<String>> {
    let session_cookie = cookie
        .get("cookie")
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
    let html = format!(
        r#"
    <!DOCTYPE html>
    <html>
        <head><title>Hello, {display_id}!</title></head>
        <body>
            <h1>Hello, {name}!</h1>
            <p>Your id is "{id:?}".</p>
        </body>
    </html>
    "#
    );
    Ok(Html(html))
}

pub fn api_routes() -> Router<AppState> {
    use axum::routing::post;
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}
