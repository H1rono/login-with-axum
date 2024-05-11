use axum::extract::{Json, State};
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
