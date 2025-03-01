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

impl crate::entity::ProvideUserRepository for State {
    type Context = sqlx::MySqlPool;
    type UserRepository = crate::repository::Repository;

    fn context(&self) -> &Self::Context {
        &self.pool
    }
    fn user_repository(&self) -> &Self::UserRepository {
        &self.repo
    }
}

impl crate::entity::ProvideUserPasswordRepository for State {
    type Context = sqlx::MySqlPool;
    type UserPasswordRepository = crate::repository::Repository;

    fn context(&self) -> &Self::Context {
        &self.pool
    }
    fn user_password_repository(&self) -> &Self::UserPasswordRepository {
        &self.repo
    }
}

impl crate::entity::ProvideCredentialManager for State {
    type Context = ();
    type CredentialManager = crate::token::Jwt;

    fn context(&self) -> &Self::Context {
        &()
    }
    fn credential_manager(&self) -> &Self::CredentialManager {
        &self.jwt
    }
}

impl State {
    pub async fn setup(&self) -> anyhow::Result<()> {
        self.repo.migrate(&self.pool).await
    }
}
