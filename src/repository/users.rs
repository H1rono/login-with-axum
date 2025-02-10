use anyhow::Context;
use sqlx::{query, query_as, Decode, Encode, FromRow, MySql, Type};
use uuid::Uuid;

use crate::model::{User, UserId};
use crate::{Elimination, Repository};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(transparent)]
pub(super) struct DbUserId(pub(super) Uuid);

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

#[derive(Debug, Clone, FromRow)]
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

impl<'a> Decode<'a, MySql> for UserId {
    fn decode(
        value: <MySql as sqlx::Database>::ValueRef<'a>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        <Uuid as Decode<'a, MySql>>::decode(value).map(UserId)
    }
}

impl<'a> Encode<'a, MySql> for UserId {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        self.0.encode_by_ref(buf)
    }
}

impl Type<MySql> for UserId {
    fn type_info() -> <MySql as sqlx::Database>::TypeInfo {
        <Uuid as Type<MySql>>::type_info()
    }

    fn compatible(ty: &<MySql as sqlx::Database>::TypeInfo) -> bool {
        <Uuid as Type<MySql>>::compatible(ty)
    }
}

#[allow(unused)]
impl Repository {
    pub async fn get_users(&self) -> Result<Vec<User>, Elimination> {
        let users = query_as("SELECT * FROM `users`")
            .fetch_all(&self.pool)
            .await
            .context("Failed to fetch users")?;
        let users = users.into_iter().map(|u: DbUser| u.into()).collect();
        Ok(users)
    }

    pub async fn get_user_by_id(&self, id: UserId) -> Result<User, Elimination> {
        let id = DbUserId::from(id);
        let user: Option<DbUser> = query_as("SELECT * FROM `users` WHERE `id` = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch user by id")?;
        let user = user.ok_or_else(|| Elimination::not_found("User not found"))?;
        Ok(user.into())
    }

    pub async fn get_user_by_display_id(&self, display_id: &str) -> Result<User, Elimination> {
        let user: Option<DbUser> = query_as("SELECT * FROM `users` WHERE `display_id` = ?")
            .bind(display_id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch user by display_id")?;
        let user = user.ok_or_else(|| Elimination::not_found("User not found"))?;
        Ok(user.into())
    }

    pub async fn create_user(&self, user: User) -> Result<(), Elimination> {
        use crate::error::RejectKind;

        match self.get_user_by_display_id(&user.display_id).await {
            Ok(_) => {
                return Err(Elimination::conflict(
                    "A user with the same display id already exists",
                ));
            }
            Err(Elimination::Reject(r)) if r.kind() == RejectKind::NotFound => {}
            Err(e) => return Err(e),
        };
        query("INSERT INTO `users` (`id`, `display_id`, `name`) VALUES (?, ?, ?)")
            .bind(user.id)
            .bind(user.display_id)
            .bind(user.name)
            .execute(&self.pool)
            .await
            .context("Failed to create user")?;
        Ok(())
    }
}
