use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod users;

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct UserId(pub Uuid);

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct User {
    pub id: UserId,
    pub display_id: String,
    pub name: String,
}
