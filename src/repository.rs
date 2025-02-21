pub mod user_passwords;
mod users;

#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct Repository;

impl Repository {
    #[tracing::instrument(skip_all)]
    pub async fn migrate(&self, pool: &sqlx::MySqlPool) -> anyhow::Result<()> {
        use anyhow::Context;

        const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

        MIGRATOR.run(pool).await.context("Failed to mgirate")
    }
}
