pub mod user_passwords;
mod users;

// TODO
#[expect(dead_code)]
const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct Impl;
