use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Builder {
    hostname: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
}

impl super::ConnectOptions {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hostname(self, value: &str) -> Self {
        let hostname = Some(value.to_string());
        Self { hostname, ..self }
    }

    pub fn port(self, value: u16) -> Self {
        let port = Some(value);
        Self { port, ..self }
    }

    pub fn username(self, value: &str) -> Self {
        let username = Some(value.to_string());
        Self { username, ..self }
    }

    pub fn password(self, value: &str) -> Self {
        let password = Some(value.to_string());
        Self { password, ..self }
    }

    pub fn database(self, value: &str) -> Self {
        let database = Some(value.to_string());
        Self { database, ..self }
    }

    pub fn build(self) -> anyhow::Result<super::ConnectOptions> {
        use anyhow::Context;

        let hostname = self.hostname.context("hostname is not set")?;
        let port = self.port.context("port is not set")?;
        let username = self.username.context("username is not set")?;
        let password = self.password.context("password is not set")?;
        let database = self.database.context("database is not set")?;
        let opts = super::ConnectOptions {
            hostname,
            port,
            username,
            password,
            database,
        };
        Ok(opts)
    }
}

impl From<super::ConnectOptions> for sqlx::mysql::MySqlConnectOptions {
    fn from(value: super::ConnectOptions) -> Self {
        let super::ConnectOptions {
            hostname,
            port,
            username,
            password,
            database,
        } = value;
        sqlx::mysql::MySqlConnectOptions::new()
            .host(&hostname)
            .port(port)
            .username(&username)
            .password(&password)
            .database(&database)
    }
}
