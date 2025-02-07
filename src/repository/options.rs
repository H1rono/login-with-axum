#[derive(Debug, Clone)]
pub struct Builder<Hostname = (), Port = (), Username = (), Password = (), Database = ()> {
    hostname: Hostname,
    port: Port,
    username: Username,
    password: Password,
    database: Database,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            hostname: (),
            port: (),
            username: (),
            password: (),
            database: (),
        }
    }
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
}

impl<Hostname, Port, Username, Password, Database>
    Builder<Hostname, Port, Username, Password, Database>
{
    pub fn hostname(self, value: &str) -> Builder<String, Port, Username, Password, Database> {
        let Self {
            hostname: _,
            port,
            username,
            password,
            database,
        } = self;
        Builder {
            hostname: value.to_string(),
            port,
            username,
            password,
            database,
        }
    }

    pub fn port(self, value: u16) -> Builder<Hostname, u16, Username, Password, Database> {
        let Self {
            hostname,
            port: _,
            username,
            password,
            database,
        } = self;
        Builder {
            hostname,
            port: value,
            username,
            password,
            database,
        }
    }

    pub fn username(self, value: &str) -> Builder<Hostname, Port, String, Password, Database> {
        let Self {
            hostname,
            port,
            username: _,
            password,
            database,
        } = self;
        Builder {
            hostname,
            port,
            username: value.to_string(),
            password,
            database,
        }
    }

    pub fn password(self, value: &str) -> Builder<Hostname, Port, Username, String, Database> {
        let Self {
            hostname,
            port,
            username,
            password: _,
            database,
        } = self;
        Builder {
            hostname,
            port,
            username,
            password: value.to_string(),
            database,
        }
    }

    pub fn database(self, value: &str) -> Builder<Hostname, Port, Username, Password, String> {
        let Self {
            hostname,
            port,
            username,
            password,
            database: _,
        } = self;
        Builder {
            hostname,
            port,
            username,
            password,
            database: value.to_string(),
        }
    }
}

impl Builder<String, u16, String, String, String> {
    pub fn build(self) -> super::ConnectOptions {
        let Self {
            hostname,
            port,
            username,
            password,
            database,
        } = self;
        super::ConnectOptions {
            hostname,
            port,
            username,
            password,
            database,
        }
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
