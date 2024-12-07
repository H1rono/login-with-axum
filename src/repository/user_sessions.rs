use super::User;
use crate::Repository;

#[allow(unused, clippy::unused_async)]
impl Repository {
    pub async fn create_session_for_user(&self, user: User) -> anyhow::Result<String> {
        anyhow::bail!("not implemented")
    }

    pub async fn load_session_from_cookie(&self, cookie: &str) -> anyhow::Result<Option<User>> {
        anyhow::bail!("not implemented")
    }

    pub async fn destroy_session_for_cookie(&self, cookie: &str) -> anyhow::Result<Option<()>> {
        anyhow::bail!("not implemented")
    }
}
