#[derive(Debug, Clone)]
pub struct RepoState {
    pool: sqlx::MySqlPool,
    repo: crate::repository::Repository,
}

impl crate::entity::ProvideUserRepository for RepoState {
    type Context<'a> = &'a sqlx::MySqlPool;
    type UserRepository<'a> = crate::repository::Repository;

    fn context(&self) -> Self::Context<'_> {
        &self.pool
    }
    fn user_repository(&self) -> &Self::UserRepository<'_> {
        &self.repo
    }
}

impl crate::entity::ProvideUserPasswordRepository for RepoState {
    type Context<'a> = &'a sqlx::MySqlPool;
    type UserPasswordRepository<'a> = crate::repository::Repository;

    fn context(&self) -> Self::Context<'_> {
        &self.pool
    }
    fn user_password_repository(&self) -> &Self::UserPasswordRepository<'_> {
        &self.repo
    }
}

impl RepoState {
    async fn setup(&self) -> anyhow::Result<()> {
        self.repo.migrate(&self.pool).await
    }
}

#[derive(Clone)]
pub struct StateInit {
    pub cookie_name: String,
    pub path_prefix: String,
    pub pool: sqlx::MySqlPool,
    pub repo: crate::repository::Repository,
    pub jwt: crate::token::Jwt,
}

#[must_use]
#[derive(Clone)]
pub struct State {
    cookie_name: String,
    path_prefix: String,
    repo: RepoState,
    jwt: crate::token::Jwt,
    registry: crate::registry::Registry,
}

impl crate::router::RouteConfig for State {
    fn cookie_name(&self) -> &str {
        &self.cookie_name
    }

    fn path_prefix(&self) -> &str {
        &self.path_prefix
    }
}

impl crate::entity::ProvideCredentialManager for State {
    type Context<'a> = ();
    type CredentialManager<'a> = crate::token::Jwt;

    fn context(&self) -> Self::Context<'_> {}
    fn credential_manager(&self) -> &Self::CredentialManager<'_> {
        &self.jwt
    }
}

impl crate::entity::ProvideUserRegistry for State {
    type Context<'a> = &'a RepoState;
    type UserRegistry<'a> = crate::registry::Registry;

    fn context(&self) -> Self::Context<'_> {
        &self.repo
    }
    fn user_registry(&self) -> &Self::UserRegistry<'_> {
        &self.registry
    }
}

impl State {
    pub fn new(init: StateInit) -> Self {
        let StateInit {
            cookie_name,
            path_prefix,
            pool,
            repo,
            jwt,
        } = init;
        let repo = RepoState { pool, repo };
        let registry = crate::registry::Registry::new();
        Self {
            cookie_name,
            path_prefix,
            repo,
            jwt,
            registry,
        }
    }

    pub async fn setup(&self) -> anyhow::Result<()> {
        self.repo.setup().await
    }
}
