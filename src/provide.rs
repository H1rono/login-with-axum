#[derive(Debug, Clone, Copy, Default)]
pub struct Impls {
    repository: crate::repository::Impl,
    jwt: crate::token::Jwt,
}

#[derive(Clone)]
pub struct State {
    // configs
    pub token_config: crate::token::JwtConfigImpl,
    pub bcrypt_cost: u32,
    pub cookie_name: String,
    pub path_prefix: String,
    // connections
    pub pool: sqlx::MySqlPool,
    // entity trait impls
    pub impls: Impls,
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
    bcrypt_cost: u32,
}

impl AsRef<sqlx::MySqlPool> for UserPasswordRepoCtx<'_> {
    fn as_ref(&self) -> &sqlx::MySqlPool {
        self.pool
    }
}

impl crate::repository::user_passwords::BcryptConfig for UserPasswordRepoCtx<'_> {
    fn bcrypt_cost(&self) -> u32 {
        self.bcrypt_cost
    }
}

impl crate::entity::ProvideUserRepository for State {
    type Context<'a> = UserRepoCtx<'a>;
    type UserRepository = crate::repository::Impl;

    fn context(&self) -> Self::Context<'_> {
        UserRepoCtx(&self.pool)
    }
    fn user_repository(&self) -> &Self::UserRepository {
        &self.impls.repository
    }
}

impl crate::entity::ProvideUserPasswordRepository for State {
    type Context<'a> = UserPasswordRepoCtx<'a>;
    type UserPasswordRepository = crate::repository::Impl;

    fn context(&self) -> Self::Context<'_> {
        UserPasswordRepoCtx {
            pool: &self.pool,
            bcrypt_cost: self.bcrypt_cost,
        }
    }
    fn user_password_repository(&self) -> &Self::UserPasswordRepository {
        &self.impls.repository
    }
}

impl crate::entity::ProvideCredentialManager for State {
    type Context<'a> = &'a crate::token::JwtConfigImpl;
    type CredentialManager = crate::token::Jwt;

    fn context(&self) -> Self::Context<'_> {
        &self.token_config
    }
    fn credential_manager(&self) -> &Self::CredentialManager {
        &self.impls.jwt
    }
}
