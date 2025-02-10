use anyhow::Context;
use axum::extract::{Form, State};
use axum::response::{IntoResponse, Redirect};
use axum::{Json, Router};
use axum_extra::extract::cookie;
use serde::{Deserialize, Serialize};

use crate::model::User;
use crate::{Elimination, Repository, TokenManager};

#[must_use]
#[derive(Clone)]
pub struct AppState {
    repository: Repository,
    token_manager: TokenManager,
    prefix: String,
}

const COOKIE_NAME: &str = "ax_session";

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

#[derive(Debug)]
pub struct ErrorResponse(Elimination);

impl From<Elimination> for ErrorResponse {
    fn from(value: Elimination) -> Self {
        ErrorResponse(value)
    }
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(value: anyhow::Error) -> Self {
        Elimination::from(value).into()
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        use crate::error::{Reject, RejectKind};

        let status_code = |r: &Reject| match r.kind() {
            RejectKind::Unauthorized => StatusCode::UNAUTHORIZED,
            RejectKind::BadRequest => StatusCode::BAD_REQUEST,
            RejectKind::NotFound => StatusCode::NOT_FOUND,
            RejectKind::Conflict => StatusCode::CONFLICT,
        };
        match self.0 {
            Elimination::Reject(r) => {
                tracing::info!("Reject: {r}");
                (status_code(&r), r.message().to_string()).into_response()
            }
            Elimination::Error(e) => {
                tracing::error!(error = ?e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub async fn register(
    State(app): State<AppState>,
    Form(req): Form<RegisterUserRequest>,
) -> Result<Redirect, ErrorResponse> {
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
) -> Result<(cookie::CookieJar, Redirect), ErrorResponse> {
    let user = app
        .repository
        .get_user_by_display_id(&req.display_id)
        .await?;
    let verification = app
        .repository
        .verify_user_password(user.id, &req.password)
        .await?;
    if !verification {
        let e = Elimination::unauthorized("unauthorized");
        return Err(e.into());
    }
    let cookie_value = app.token_manager.encode(user.id)?;
    let cookie = cookie::Cookie::build((COOKIE_NAME, cookie_value))
        .path(app.prefix.clone())
        .http_only(true)
        .build();
    let cookie_jar = cookie_jar.add(cookie);
    Ok((cookie_jar, Redirect::to(&format!("{}me.html", app.prefix))))
}

pub async fn logout(
    State(app): State<AppState>,
    cookie_jar: cookie::CookieJar,
) -> Result<(cookie::CookieJar, Redirect), ErrorResponse> {
    let _cookie = cookie_jar
        .get(COOKIE_NAME)
        .ok_or_else(|| Elimination::unauthorized("Unauthorized"))?;
    // TODO Expire within TokenManager
    // TODO: add attribute `Expires` with chrono
    let cookie = cookie::Cookie::build(COOKIE_NAME)
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
) -> Result<Json<User>, ErrorResponse> {
    let session_cookie = cookie_jar
        .get(COOKIE_NAME)
        .context("Unauthenticated")?
        .value();
    let user_id = app.token_manager.decode(session_cookie)?;
    let user = app.repository.get_user_by_id(user_id).await?;
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
