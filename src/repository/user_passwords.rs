use anyhow::Context;

use super::users::DbUserId;
use crate::error::Failure;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(transparent)]
struct DbPsk(String);

#[derive(Debug, Clone, sqlx::FromRow)]
struct DbUserPassword {
    #[sqlx(rename = "user_id")]
    id: DbUserId,
    psk: DbPsk,
}

#[must_use]
pub trait BcryptConfig: Send + Sync {
    fn bcrypt_cost(&self) -> u32;
}

impl<Context> crate::entity::UserPasswordRepository<Context> for super::Impl
where
    Context: BcryptConfig + AsRef<sqlx::MySqlPool>,
{
    async fn save_user_password(
        &self,
        ctx: Context,
        params: crate::entity::SaveUserPasswordParams,
    ) -> Result<(), Failure> {
        let psk = bcrypt::hash(params.raw, ctx.bcrypt_cost()).context("Failed to hash password")?;
        let password = DbUserPassword {
            id: params.user_id.into(),
            psk: DbPsk(psk),
        };
        sqlx::query("INSERT INTO `user_passwords` (`user_id`, `psk`) VALUES (?, ?)")
            .bind(password.id)
            .bind(password.psk)
            .execute(ctx.as_ref())
            .await
            .context("Failed to insert user password")?;
        Ok(())
    }

    async fn verify_user_password(
        &self,
        ctx: Context,
        params: crate::entity::VerifyUserPasswordParams,
    ) -> Result<bool, Failure> {
        let DbPsk(psk) = sqlx::query_as("SELECT * FROM `user_passwords` WHERE `user_id` = ?")
            .bind(DbUserId::from(params.user_id))
            .fetch_optional(ctx.as_ref())
            .await
            .context("Failed to get user password")?
            .map(|p: DbUserPassword| p.psk)
            .ok_or_else(|| Failure::not_found("password not found"))?;
        // TODO: log if err
        let res = bcrypt::verify(params.raw, &psk).context("Failed to challenge bcrypt hash")?;
        Ok(res)
    }
}
