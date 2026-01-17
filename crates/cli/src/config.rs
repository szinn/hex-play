use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    /// (required) Fully qualified URL for accessing Postgres server.
    /// e.g. postgres://user:password@host/database
    pub database_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
}

impl Config {
    pub fn load() -> Result<Config, Error> {
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("HPLAY").try_parsing(true).separator("__"))
            .build()?;

        let config: Config = config.try_deserialize()?;

        Ok(config)
    }
}
