#[derive(Debug, Clone)]
pub struct RepoState {
    pool: sqlx::MySqlPool,
    repo: crate::repository::Repository,
}

impl crate::entity::ProvideUserRepository for RepoState {
    type Context = sqlx::MySqlPool;
    type UserRepository = crate::repository::Repository;

    fn context(&self) -> &Self::Context {
        &self.pool
    }
    fn user_repository(&self) -> &Self::UserRepository {
        &self.repo
    }
}

impl crate::entity::ProvideUserPasswordRepository for RepoState {
    type Context = sqlx::MySqlPool;
    type UserPasswordRepository = crate::repository::Repository;

    fn context(&self) -> &Self::Context {
        &self.pool
    }
    fn user_password_repository(&self) -> &Self::UserPasswordRepository {
        &self.repo
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

impl crate::entity::ProvideUserRepository for State {
    type Context = sqlx::MySqlPool;
    type UserRepository = crate::repository::Repository;

    fn context(&self) -> &Self::Context {
        &self.repo.pool
    }
    fn user_repository(&self) -> &Self::UserRepository {
        &self.repo.repo
    }
}

impl crate::entity::ProvideUserPasswordRepository for State {
    type Context = sqlx::MySqlPool;
    type UserPasswordRepository = crate::repository::Repository;

    fn context(&self) -> &Self::Context {
        &self.repo.pool
    }
    fn user_password_repository(&self) -> &Self::UserPasswordRepository {
        &self.repo.repo
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

impl crate::entity::ProvideUserRegistry for State {
    type Context = RepoState;
    type UserRegistry = crate::registry::Registry;

    fn context(&self) -> &Self::Context {
        &self.repo
    }
    fn user_registry(&self) -> &crate::registry::Registry {
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
        self.repo.repo.migrate(&self.repo.pool).await
    }
}
