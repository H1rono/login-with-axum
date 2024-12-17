use anyhow::Context;
use sqlx::{query, query_as, FromRow, Type};

use super::users::DbUserId;
use crate::model::{UserId, UserPassword};
use crate::Repository;

#[derive(Debug, Clone, Type)]
#[sqlx(transparent)]
struct DbPsk(String);

impl From<DbPsk> for UserPassword {
    fn from(value: DbPsk) -> Self {
        Self(value.0)
    }
}

impl From<UserPassword> for DbPsk {
    fn from(value: UserPassword) -> Self {
        Self(value.0)
    }
}

#[derive(Debug, Clone, FromRow)]
struct DbUserPassword {
    #[sqlx(rename = "user_id")]
    id: DbUserId,
    psk: DbPsk,
}

#[allow(unused)]
impl Repository {
    pub(super) async fn get_user_password_by_id(
        &self,
        id: UserId,
    ) -> sqlx::Result<Option<UserPassword>> {
        let user_password: Option<DbUserPassword> =
            query_as("SELECT * FROM `user_passwords` WHERE `user_id` = ?")
                .bind(DbUserId::from(id))
                .fetch_optional(&self.pool)
                .await?;
        let user_password = user_password.map(|p| p.psk.into());
        Ok(user_password)
    }

    async fn write_user_password(&self, password: DbUserPassword) -> sqlx::Result<()> {
        query("INSERT INTO `user_passwords` (`user_id`, `psk`) VALUES (?, ?)")
            .bind(password.id)
            .bind(password.psk)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save_raw_password(&self, id: UserId, raw: &str) -> anyhow::Result<()> {
        let psk = bcrypt::hash(raw, self.bcrypt_cost).with_context(|| "failed to hash password")?;
        let password = DbUserPassword {
            id: id.into(),
            psk: DbPsk(psk),
        };
        self.write_user_password(password).await?;
        Ok(())
    }

    pub async fn verify_user_password(&self, id: UserId, raw: &str) -> sqlx::Result<Option<bool>> {
        let user_password = self.get_user_password_by_id(id).await?;
        let res = user_password.map(|password| {
            // TODO: log if err
            bcrypt::verify(raw, &password.0).is_ok_and(|p| p)
        });
        Ok(res)
    }
}
