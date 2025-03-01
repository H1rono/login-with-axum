use anyhow::Context;

use crate::Failure;
use crate::entity::{User, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
pub(super) struct DbUserId(pub(super) uuid::Uuid);

impl From<UserId> for DbUserId {
    fn from(value: UserId) -> Self {
        Self(value.0)
    }
}

impl From<DbUserId> for UserId {
    fn from(value: DbUserId) -> Self {
        Self(value.0)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(super) struct DbUser {
    pub(super) id: DbUserId,
    pub(super) display_id: String,
    pub(super) name: String,
}

impl From<DbUser> for User {
    fn from(value: DbUser) -> Self {
        let DbUser {
            id,
            display_id,
            name,
        } = value;
        Self {
            id: id.into(),
            display_id,
            name,
        }
    }
}

impl<Context> crate::entity::UserRepository<Context> for super::Repository
where
    Context: super::AsMySqlPool,
{
    async fn get_users(&self, ctx: Context) -> Result<Vec<User>, Failure> {
        let users = sqlx::query_as("SELECT * FROM `users`")
            .fetch_all(ctx.as_mysql_pool())
            .await
            .context("Failed to fetch users")?
            .into_iter()
            .map(|u: DbUser| u.into())
            .collect();
        Ok(users)
    }

    async fn get_user(
        &self,
        ctx: Context,
        params: crate::entity::GetUserParams,
    ) -> Result<User, Failure> {
        use crate::entity::GetUserParams::{ByDisplayId, ById};

        let pool = ctx.as_mysql_pool();
        match params {
            ById(id) => self.get_user_by_id(pool, id).await,
            ByDisplayId(display_id) => self.get_user_by_display_id(pool, &display_id).await,
        }
    }

    async fn create_user(
        &self,
        ctx: Context,
        params: crate::entity::CreateUserParams,
    ) -> Result<User, Failure> {
        use crate::error::RejectKind;

        let pool = ctx.as_mysql_pool();
        match self.get_user_by_display_id(pool, &params.display_id).await {
            Ok(_) => {
                return Err(Failure::conflict(
                    "A user with the same display id already exists",
                ));
            }
            Err(Failure::Reject(r)) if r.kind() == RejectKind::NotFound => {}
            Err(e) => return Err(e),
        };
        let id = DbUserId(uuid::Uuid::new_v4());
        let crate::entity::CreateUserParams { display_id, name } = params;
        sqlx::query("INSERT INTO `users` (`id`, `display_id`, `name`) VALUES (?, ?, ?)")
            .bind(id)
            .bind(display_id)
            .bind(name)
            .execute(pool)
            .await
            .context("Failed to create user")?;
        let user = self.get_user_by_id(pool, id.into()).await?;
        Ok(user)
    }
}

impl super::Repository {
    async fn get_user_by_id(&self, pool: &sqlx::MySqlPool, id: UserId) -> Result<User, Failure> {
        let id = DbUserId::from(id);
        let user = sqlx::query_as::<_, DbUser>("SELECT * FROM `users` WHERE `id` = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
            .context("Failed to fetch user by id")?
            .ok_or_else(|| Failure::not_found("User not found"))?;
        Ok(user.into())
    }

    async fn get_user_by_display_id(
        &self,
        pool: &sqlx::MySqlPool,
        display_id: &str,
    ) -> Result<User, Failure> {
        let user = sqlx::query_as::<_, DbUser>("SELECT * FROM `users` WHERE `display_id` = ?")
            .bind(display_id)
            .fetch_optional(pool)
            .await
            .context("Failed to fetch user by display_id")?
            .ok_or_else(|| Failure::not_found("User not found"))?;
        Ok(user.into())
    }
}
