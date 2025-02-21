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
