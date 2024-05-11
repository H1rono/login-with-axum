use anyhow::Context;
use sqlx::{query, query_as};

use super::{UserId, UserPassword};
use crate::Repository;

#[allow(unused)]
impl Repository {
    pub(super) async fn get_user_password_by_id(
        &self,
        id: UserId,
    ) -> sqlx::Result<Option<UserPassword>> {
        query_as("SELECT * FROM `user_passwords` WHERE `id` = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub(super) async fn write_user_password(&self, password: UserPassword) -> sqlx::Result<()> {
        query("INSERT INTO `user_passwords` (`user_id`, `psk`) VALUES (?, ?)")
            .bind(password.id)
            .bind(password.psk)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save_raw_password(&self, id: UserId, raw: &str) -> anyhow::Result<()> {
        let psk = bcrypt::hash(raw, self.bcrypt_cost).with_context(|| "failed to hash password")?;
        let password = UserPassword { id, psk };
        self.write_user_password(password).await?;
        Ok(())
    }

    pub async fn verify_user_password(&self, id: UserId, raw: &str) -> sqlx::Result<Option<bool>> {
        let user_password = self.get_user_password_by_id(id).await?;
        let res = user_password.map(|password| {
            // TODO: log if err
            bcrypt::verify(raw, &password.psk).is_ok_and(|p| p)
        });
        Ok(res)
    }
}
