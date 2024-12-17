use serde::{Deserialize, Serialize};
use sqlx::{mysql, FromRow};

use crate::model::UserId;
use crate::Repository;

mod options;
mod user_passwords;
mod user_sessions;
mod users;

const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub type SessionStore = ();

#[allow(unused)]
impl Repository {
    pub async fn connect_with(options: ConnectOptions) -> sqlx::Result<Self> {
        let pool = mysql::MySqlPool::connect_with(options.into()).await?;
        // let session_store =
        //     MySqlSessionStore::from_client(pool.clone()).with_table_name("user_sessions");
        let session_store = ();
        Ok(Self {
            pool,
            session_store,
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
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct UserPassword {
    #[serde(rename = "user_id")]
    #[sqlx(rename = "user_id")]
    pub id: UserId,
    pub psk: String,
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ConnectOptions {
    hostname: String,
    port: u16,
    username: String,
    password: String,
    database: String,
}
