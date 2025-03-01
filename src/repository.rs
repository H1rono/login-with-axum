pub mod user_passwords;
mod users;

#[must_use]
#[derive(Debug, Clone)]
pub struct Repository {
    bcrypt_cost: u32,
}

impl Repository {
    pub fn new(bcrypt_cost: u32) -> Self {
        Self { bcrypt_cost }
    }

    #[tracing::instrument(skip_all)]
    pub async fn migrate(&self, pool: &sqlx::MySqlPool) -> anyhow::Result<()> {
        use anyhow::Context;

        const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

        MIGRATOR.run(pool).await.context("Failed to mgirate")
    }
}

pub trait AsMySqlPool: Send + Sync {
    fn as_mysql_pool(&self) -> &sqlx::MySqlPool;
}

impl AsMySqlPool for sqlx::MySqlPool {
    fn as_mysql_pool(&self) -> &sqlx::MySqlPool {
        self
    }
}

impl<T> AsMySqlPool for &T
where
    T: AsMySqlPool,
{
    fn as_mysql_pool(&self) -> &sqlx::MySqlPool {
        T::as_mysql_pool(self)
    }
}
