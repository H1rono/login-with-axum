use anyhow::anyhow;
use sqlx::{query, query_as, Decode, Encode, MySql, Type};
use uuid::Uuid;

use super::{User, UserId};
use crate::Repository;

impl UserId {
    pub fn new(inner: Uuid) -> Self {
        Self(inner)
    }
}

impl From<Uuid> for UserId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<UserId> for Uuid {
    fn from(value: UserId) -> Self {
        value.0
    }
}

impl<'a> Decode<'a, MySql> for UserId {
    fn decode(
        value: <MySql as sqlx::database::HasValueRef<'a>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        <Uuid as Decode<'a, MySql>>::decode(value).map(UserId)
    }
}

impl<'a> Encode<'a, MySql> for UserId {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::database::HasArguments<'a>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.0.encode_by_ref(buf)
    }

    fn encode(
        self,
        buf: &mut <MySql as sqlx::database::HasArguments<'a>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull
    where
        Self: Sized,
    {
        self.0.encode(buf)
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
    pub async fn get_users(&self) -> sqlx::Result<Vec<User>> {
        query_as("SELECT * FROM `users`")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_user_by_id(&self, id: UserId) -> sqlx::Result<Option<User>> {
        query_as("SELECT * FROM `users` WHERE `id` = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_user_by_display_id(&self, display_id: &str) -> sqlx::Result<Option<User>> {
        query_as("SELECT * FROM `users` WHERE `display_id` = ?")
            .bind(display_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn create_user(&self, user: User) -> anyhow::Result<()> {
        if self
            .get_user_by_display_id(&user.display_id)
            .await?
            .is_some()
        {
            return Err(anyhow!("A user with the same display id already exists"));
        }
        query("INSERT INTO `users` (`id`, `display_id`, `name`) VALUES (?, ?, ?)")
            .bind(user.id)
            .bind(user.display_id)
            .bind(user.name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
