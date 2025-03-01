use std::sync::Arc;

use anyhow::Context;
use axum::extract::{Form, Json};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::cookie;
use serde::{Deserialize, Serialize};

use crate::{Failure, entity};

pub trait RouteConfig: Send + Sync {
    fn cookie_name(&self) -> &str;
    fn path_prefix(&self) -> &str;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterUserRequest {
    pub display_id: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginUserRequest {
    pub display_id: String,
    pub password: String,
}

#[derive(Debug)]
pub struct ErrorResponse(Failure);

impl From<Failure> for ErrorResponse {
    fn from(value: Failure) -> Self {
        ErrorResponse(value)
    }
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(value: anyhow::Error) -> Self {
        Failure::from(value).into()
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
            Failure::Reject(r) => {
                tracing::info!("Reject: {r}");
                (status_code(&r), r.message().to_string()).into_response()
            }
            Failure::Error(e) => {
                tracing::error!(error = ?e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

struct AppState<S>(Arc<S>);

impl<S> std::ops::Deref for AppState<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> std::clone::Clone for AppState<S> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<S> AppState<S>
where
    S: entity::ProvideUserRegistry + entity::ProvideCredentialManager + RouteConfig + 'static,
{
    async fn register(
        self,
        Form(req): Form<RegisterUserRequest>,
    ) -> Result<Redirect, ErrorResponse> {
        // TODO: validation
        let RegisterUserRequest {
            display_id,
            name,
            password,
        } = req;
        let params = entity::RegisterUserParams {
            display_id,
            name,
            raw_password: password,
        };
        let _user = self.register_user(params).await?;
        let login_path = format!("{}login.html", &self.path_prefix());
        Ok(Redirect::to(&login_path))
    }

    async fn login(
        self,
        cookie_jar: cookie::CookieJar,
        Form(req): Form<LoginUserRequest>,
    ) -> Result<(cookie::CookieJar, Redirect), ErrorResponse> {
        let params = entity::GetUserParams::ByDisplayId(req.display_id);
        let user = self.get_user(params).await?;
        let params = entity::VerifyUserPasswordParams {
            user_id: user.id,
            raw: req.password,
        };
        let verification = self.verify_user_password(params).await?;
        if !verification {
            let e = Failure::unauthorized("unauthorized");
            return Err(e.into());
        }
        let params = entity::MakeCredentialParams { user_id: user.id };
        let entity::Credential(cookie_value) = self.make_credential(params).await?;
        let prefix = self.path_prefix();
        let cookie = cookie::Cookie::build((self.cookie_name().to_string(), cookie_value))
            .path(prefix.to_string())
            .http_only(true)
            .build();
        let cookie_jar = cookie_jar.add(cookie);
        Ok((cookie_jar, Redirect::to(&format!("{prefix}me.html"))))
    }

    #[expect(clippy::unused_async)]
    async fn logout(
        self,
        cookie_jar: cookie::CookieJar,
    ) -> Result<(cookie::CookieJar, Redirect), ErrorResponse> {
        let cookie_name = self.cookie_name();
        let cookie_value = cookie_jar
            .get(cookie_name)
            .ok_or_else(|| Failure::unauthorized("Unauthorized"))?
            .value();
        let _credential = entity::Credential(cookie_value.to_string());
        // TODO: self.revoke_credential(credential).await?;
        let prefix = self.path_prefix();
        let cookie = cookie::Cookie::build(cookie_name.to_string())
            .removal()
            .path(prefix.to_string())
            .http_only(true)
            .build();
        let cookie_jar = cookie_jar.add(cookie);
        Ok((cookie_jar, Redirect::to(prefix)))
    }

    async fn me(self, cookie_jar: cookie::CookieJar) -> Result<Json<entity::User>, ErrorResponse> {
        let session_cookie = cookie_jar
            .get(self.cookie_name())
            .context("Unauthenticated")?
            .value();
        let credential = entity::Credential(session_cookie.to_string());
        let user_id = self.check_credential(credential).await?;
        // let user_id = app.token_manager.decode(session_cookie)?;
        let params = entity::GetUserParams::ById(user_id);
        let user = self.get_user(params).await?;
        Ok(Json(user))
    }

    fn router() -> axum::Router<Self> {
        use axum::extract::State;
        use axum::routing::{get, post};

        let register = |State(state): State<Self>, req| state.register(req);
        let login = |State(state): State<Self>, cookie_jar, req| state.login(cookie_jar, req);
        let logout = |State(state): State<Self>, cookie_jar| state.logout(cookie_jar);
        let me = |State(state): State<Self>, cookie_jar| state.me(cookie_jar);
        axum::Router::new()
            .route("/register", post(register))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/me", get(me))
    }
}

pub fn make<S>(state: Arc<S>) -> axum::Router
where
    S: entity::ProvideUserRegistry + entity::ProvideCredentialManager + RouteConfig + 'static,
{
    use tower_http::services::ServeDir;

    let state = AppState(state);
    let inner = axum::Router::new()
        .route("/ping", axum::routing::get(|| async { "pong" }))
        .nest("/api", AppState::<S>::router())
        .fallback_service(ServeDir::new("./dist"));
    let prefix = state.path_prefix();
    let router = if prefix == "/" {
        inner
    } else {
        axum::Router::new().nest(prefix, inner)
    };
    router.with_state(state)
}
