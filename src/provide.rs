#[must_use]
#[derive(Clone)]
pub struct State {
    // configs
    pub cookie_name: String,
    pub path_prefix: String,
    // connections
    pub pool: sqlx::MySqlPool,
    // entity trait impls
    pub repo: crate::repository::Repository,
    pub jwt: crate::token::Jwt,
}

impl crate::router::RouteConfig for State {
    fn cookie_name(&self) -> &str {
        &self.cookie_name
    }

    fn path_prefix(&self) -> &str {
        &self.path_prefix
    }
}

pub struct UserRepoCtx<'a>(&'a sqlx::MySqlPool);

impl AsRef<sqlx::MySqlPool> for UserRepoCtx<'_> {
    fn as_ref(&self) -> &sqlx::MySqlPool {
        self.0
    }
}

pub struct UserPasswordRepoCtx<'a> {
    pool: &'a sqlx::MySqlPool,
}

impl AsRef<sqlx::MySqlPool> for UserPasswordRepoCtx<'_> {
    fn as_ref(&self) -> &sqlx::MySqlPool {
        self.pool
    }
}

impl crate::entity::ProvideUserRepository for State {
    type Context<'a> = UserRepoCtx<'a>;
    type UserRepository = crate::repository::Repository;

    fn context(&self) -> Self::Context<'_> {
        UserRepoCtx(&self.pool)
    }
    fn user_repository(&self) -> &Self::UserRepository {
        &self.repo
    }
}

impl crate::entity::ProvideUserPasswordRepository for State {
    type Context<'a> = UserPasswordRepoCtx<'a>;
    type UserPasswordRepository = crate::repository::Repository;

    fn context(&self) -> Self::Context<'_> {
        UserPasswordRepoCtx { pool: &self.pool }
    }
    fn user_password_repository(&self) -> &Self::UserPasswordRepository {
        &self.repo
    }
}

impl crate::entity::ProvideCredentialManager for State {
    type Context<'a> = ();
    type CredentialManager = crate::token::Jwt;

    fn context(&self) -> Self::Context<'_> {}
    fn credential_manager(&self) -> &Self::CredentialManager {
        &self.jwt
    }
}

impl State {
    pub async fn setup(&self) -> anyhow::Result<()> {
        self.repo.migrate(&self.pool).await
    }
}
