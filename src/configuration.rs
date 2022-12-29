//! src/configuration.rs

use std::env;

use config::Config;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}
#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // initializing our configuration reader and then
    // add configuration values from a file named "configuration".
    // it will look for any top-level file with and extention
    //that "config" knows how to parse: yaml, json, etc.
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    let app_environment = env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".into());

    Config::builder()
        .add_source(config::File::from(configuration_directory.join("base")))
        .add_source(config::File::from(
            configuration_directory.join(app_environment),
        ))
        .add_source(config::Environment::with_prefix("R2P"))
        .build()?
        .try_deserialize()
}
