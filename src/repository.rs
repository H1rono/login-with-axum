use async_sqlx_session::MySqlSessionStore;
use serde::{Deserialize, Serialize};
use sqlx::{mysql, FromRow};
use uuid::Uuid;

use crate::Repository;

mod user_passwords;
mod user_sessions;
mod users;

const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[allow(unused)]
impl Repository {
    async fn connect_with(options: mysql::MySqlConnectOptions) -> sqlx::Result<Self> {
        let pool = mysql::MySqlPool::connect_with(options).await?;
        let session_store =
            MySqlSessionStore::from_client(pool.clone()).with_table_name("user_sessions");
        Ok(Self {
            pool,
            session_store,
            bcrypt_cost: bcrypt::DEFAULT_COST,
        })
    }

    async fn migrate(&self) -> sqlx::Result<()> {
        MIGRATOR.run(&self.pool).await?;
        self.session_store.migrate().await?;
        Ok(())
    }
}

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct UserId(Uuid);

#[must_use]
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct User {
    pub id: UserId,
    pub display_id: String,
    pub name: String,
}

#[must_use]
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct UserPassword {
    #[serde(rename = "user_id")]
    pub id: UserId,
    pub psk: String,
}
