use anyhow::Context;
use async_session::{Session, SessionStore};

use super::User;
use crate::Repository;

#[allow(unused)]
impl Repository {
    pub async fn create_session_for_user(&self, user: User) -> anyhow::Result<String> {
        let mut session = Session::new();
        session
            .insert("user", user)
            .with_context(|| "failed to insert user into session")?;
        let res = self
            .session_store
            .store_session(session)
            .await
            .with_context(|| "failed to save session to database")?
            .with_context(|| "unexpected error while converting session to cookie")?;
        Ok(res)
    }

    pub async fn load_session_from_cookie(&self, cookie: &str) -> anyhow::Result<Option<User>> {
        let session = self.session_store.load_session(cookie.to_string()).await?;
        Ok(session.and_then(|s| s.get("user")))
    }
}
