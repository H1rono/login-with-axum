use crate::entity;
use crate::error::Failure;

#[must_use]
#[derive(Debug, Clone)]
pub struct Registry {
    _private: (),
}

impl Registry {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl<Context> entity::UserRegistry<Context> for Registry
where
    Context: entity::ProvideUserRepository + entity::ProvideUserPasswordRepository,
{
    async fn get_user(
        &self,
        ctx: &Context,
        params: entity::GetUserParams,
    ) -> Result<entity::User, Failure> {
        ctx.get_user(params).await
    }

    async fn get_users(&self, ctx: &Context) -> Result<Vec<entity::User>, Failure> {
        ctx.get_users().await
    }

    async fn register_user(
        &self,
        ctx: &Context,
        params: entity::RegisterUserParams,
    ) -> Result<entity::User, Failure> {
        let entity::RegisterUserParams {
            display_id,
            name,
            raw_password: raw,
        } = params;
        let params = entity::CreateUserParams { display_id, name };
        let user = ctx.create_user(params).await?;
        let params = entity::SaveUserPasswordParams {
            user_id: user.id,
            raw,
        };
        ctx.save_user_password(params).await?;
        Ok(user)
    }

    async fn update_user_password(
        &self,
        ctx: &Context,
        params: entity::UpdateUserPasswordParams,
    ) -> Result<(), Failure> {
        let entity::UpdateUserPasswordParams {
            user_id,
            new_raw: raw,
        } = params;
        let params = entity::SaveUserPasswordParams { user_id, raw };
        ctx.save_user_password(params).await
    }

    async fn verify_user_password(
        &self,
        ctx: &Context,
        params: entity::VerifyUserPasswordParams,
    ) -> Result<bool, Failure> {
        ctx.verify_user_password(params).await
    }
}
