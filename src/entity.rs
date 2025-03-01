use serde::{Deserialize, Serialize};

use crate::error::Failure;

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct UserId(pub uuid::Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct User {
    pub id: UserId,
    pub display_id: String,
    pub name: String,
}

// MARK: UserRepository

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GetUserParams {
    ById(UserId),
    ByDisplayId(String),
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct CreateUserParams {
    pub display_id: String,
    pub name: String,
}

#[must_use]
pub trait UserRepository<Context>: Send + Sync {
    fn get_users(&self, ctx: &Context) -> impl Future<Output = Result<Vec<User>, Failure>> + Send;
    fn get_user(
        &self,
        ctx: &Context,
        params: GetUserParams,
    ) -> impl Future<Output = Result<User, Failure>> + Send;
    fn create_user(
        &self,
        ctx: &Context,
        params: CreateUserParams,
    ) -> impl Future<Output = Result<User, Failure>> + Send;
}

#[must_use]
pub trait ProvideUserRepository: Send + Sync {
    type Context;
    type UserRepository: UserRepository<Self::Context>;

    fn context(&self) -> &Self::Context;
    fn user_repository(&self) -> &Self::UserRepository;

    fn get_users(&self) -> impl Future<Output = Result<Vec<User>, Failure>> + Send {
        let ctx = self.context();
        self.user_repository().get_users(ctx)
    }
    fn get_user(
        &self,
        params: GetUserParams,
    ) -> impl Future<Output = Result<User, Failure>> + Send {
        let ctx = self.context();
        self.user_repository().get_user(ctx, params)
    }
    fn create_user(
        &self,
        params: CreateUserParams,
    ) -> impl Future<Output = Result<User, Failure>> + Send {
        let ctx = self.context();
        self.user_repository().create_user(ctx, params)
    }
}

// MARK: UserPasswordRepository

#[must_use]
pub struct SaveUserPasswordParams {
    pub user_id: UserId,
    pub raw: String,
}

#[must_use]
pub struct VerifyUserPasswordParams {
    pub user_id: UserId,
    pub raw: String,
}

#[must_use]
pub trait UserPasswordRepository<Context>: Send + Sync {
    fn save_user_password(
        &self,
        ctx: &Context,
        params: SaveUserPasswordParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send;
    fn verify_user_password(
        &self,
        ctx: &Context,
        params: VerifyUserPasswordParams,
    ) -> impl Future<Output = Result<bool, Failure>> + Send;
}

#[must_use]
pub trait ProvideUserPasswordRepository: Send + Sync {
    type Context;
    type UserPasswordRepository: UserPasswordRepository<Self::Context>;

    fn context(&self) -> &Self::Context;
    fn user_password_repository(&self) -> &Self::UserPasswordRepository;

    fn save_user_password(
        &self,
        params: SaveUserPasswordParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        let ctx = self.context();
        self.user_password_repository()
            .save_user_password(ctx, params)
    }
    fn verify_user_password(
        &self,
        params: VerifyUserPasswordParams,
    ) -> impl Future<Output = Result<bool, Failure>> + Send {
        let ctx = self.context();
        self.user_password_repository()
            .verify_user_password(ctx, params)
    }
}

// MARK: CredentialManager

#[must_use]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Credential(pub String);

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MakeCredentialParams {
    pub user_id: UserId,
}

#[must_use]
pub trait CredentialManager<Context>: Send + Sync {
    fn make_credential(
        &self,
        ctx: &Context,
        params: MakeCredentialParams,
    ) -> impl Future<Output = Result<Credential, Failure>> + Send;
    fn revoke_credential(
        &self,
        ctx: &Context,
        credential: Credential,
    ) -> impl Future<Output = Result<(), Failure>> + Send;
    fn check_credential(
        &self,
        ctx: &Context,
        credential: Credential,
    ) -> impl Future<Output = Result<UserId, Failure>> + Send;
}

#[must_use]
pub trait ProvideCredentialManager: Send + Sync {
    type Context;
    type CredentialManager: CredentialManager<Self::Context>;

    fn context(&self) -> &Self::Context;
    fn credential_manager(&self) -> &Self::CredentialManager;

    fn make_credential(
        &self,
        params: MakeCredentialParams,
    ) -> impl Future<Output = Result<Credential, Failure>> + Send {
        let ctx = self.context();
        self.credential_manager().make_credential(ctx, params)
    }
    fn revoke_credential(
        &self,
        credential: Credential,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        let ctx = self.context();
        self.credential_manager().revoke_credential(ctx, credential)
    }
    fn check_credential(
        &self,
        credential: Credential,
    ) -> impl Future<Output = Result<UserId, Failure>> + Send {
        let ctx = self.context();
        self.credential_manager().check_credential(ctx, credential)
    }
}

// MARK: UserRegistry

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct RegisterUserParams {
    pub display_id: String,
    pub name: String,
    pub raw_password: String,
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct UpdateUserPasswordParams {
    pub user_id: UserId,
    #[serde(rename = "password")]
    pub new_raw: String,
}

#[must_use]
pub trait UserRegistry<Context>: Send + Sync {
    fn get_user(
        &self,
        ctx: &Context,
        params: GetUserParams,
    ) -> impl Future<Output = Result<User, Failure>> + Send;
    fn get_users(&self, ctx: &Context) -> impl Future<Output = Result<Vec<User>, Failure>> + Send;
    fn register_user(
        &self,
        ctx: &Context,
        params: RegisterUserParams,
    ) -> impl Future<Output = Result<User, Failure>> + Send;
    fn verify_user_password(
        &self,
        ctx: &Context,
        params: VerifyUserPasswordParams,
    ) -> impl Future<Output = Result<bool, Failure>> + Send;
    fn update_user_password(
        &self,
        ctx: &Context,
        params: UpdateUserPasswordParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send;
}

#[must_use]
pub trait ProvideUserRegistry: Send + Sync {
    type Context;
    type UserRegistry: UserRegistry<Self::Context>;

    fn context(&self) -> &Self::Context;
    fn user_registry(&self) -> &Self::UserRegistry;

    fn get_user(
        &self,
        params: GetUserParams,
    ) -> impl Future<Output = Result<User, Failure>> + Send {
        let ctx = self.context();
        self.user_registry().get_user(ctx, params)
    }
    fn get_users(&self) -> impl Future<Output = Result<Vec<User>, Failure>> + Send {
        let ctx = self.context();
        self.user_registry().get_users(ctx)
    }
    fn register_user(
        &self,
        params: RegisterUserParams,
    ) -> impl Future<Output = Result<User, Failure>> + Send {
        let ctx = self.context();
        self.user_registry().register_user(ctx, params)
    }
    fn verify_user_password(
        &self,
        params: VerifyUserPasswordParams,
    ) -> impl Future<Output = Result<bool, Failure>> + Send {
        let ctx = self.context();
        self.user_registry().verify_user_password(ctx, params)
    }
    fn update_user_password(
        &self,
        params: UpdateUserPasswordParams,
    ) -> impl Future<Output = Result<(), Failure>> + Send {
        let ctx = self.context();
        self.user_registry().update_user_password(ctx, params)
    }
}
