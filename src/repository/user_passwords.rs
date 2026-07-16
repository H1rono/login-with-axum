use anyhow::Context;

use super::users::DbUserId;
use crate::error::Failure;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(transparent)]
struct DbPsk(String);

impl From<String> for DbPsk {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct DbUserPassword {
    #[expect(unused)]
    #[sqlx(rename = "user_id")]
    id: DbUserId,
    psk: DbPsk,
}

impl<Context> crate::entity::UserPasswordRepository<Context> for super::Repository
where
    Context: super::AsMySqlPool,
{
    async fn save_user_password(
        &self,
        ctx: Context,
        params: crate::entity::SaveUserPasswordParams,
    ) -> Result<(), Failure> {
        let crate::entity::SaveUserPasswordParams { user_id, raw } = params;
        let psk = bcrypt::hash(raw, self.bcrypt_cost).context("Failed to hash password")?;
        sqlx::query_file!(
            "src/repository/user_passwords/save_user_password.sql",
            user_id.0,
            psk
        )
        .execute(ctx.as_mysql_pool())
        .await
        .context("Failed to insert user password")?;
        Ok(())
    }

    async fn verify_user_password(
        &self,
        ctx: Context,
        params: crate::entity::VerifyUserPasswordParams,
    ) -> Result<bool, Failure> {
        let crate::entity::VerifyUserPasswordParams { user_id, raw } = params;
        let DbPsk(psk) = sqlx::query_file_as!(
            DbUserPassword,
            "src/repository/user_passwords/verify_user_password.sql",
            user_id.0
        )
        .fetch_optional(ctx.as_mysql_pool())
        .await
        .context("Failed to get user password")?
        .map(|p| p.psk)
        .ok_or_else(|| Failure::not_found("password not found"))?;
        // TODO: log if err
        let res = bcrypt::verify(raw, &psk).context("Failed to challenge bcrypt hash")?;
        Ok(res)
    }
}
