use serde::{Deserialize, Serialize};
use sqlx::mysql;

mod options;
mod user_passwords;
mod users;

const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[must_use]
#[derive(Debug, Clone)]
pub struct Repository {
    pool: mysql::MySqlPool,
    bcrypt_cost: u32,
}

#[allow(unused)]
impl Repository {
    pub async fn connect_with(options: ConnectOptions) -> sqlx::Result<Self> {
        let pool = mysql::MySqlPool::connect_with(options.into()).await?;
        Ok(Self {
            pool,
            bcrypt_cost: bcrypt::DEFAULT_COST,
        })
    }

    pub async fn migrate(&self) -> sqlx::Result<()> {
        MIGRATOR.run(&self.pool).await?;
        // self.session_store.migrate().await?;
        Ok(())
    }
}

#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct Impl;

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ConnectOptions {
    hostname: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}
