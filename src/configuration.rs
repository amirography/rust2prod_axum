//! src/configuration.rs

use config::Config;
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
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
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // initializing our configuration reader and then
    // add configuration values from a file named "configuration".
    // it will look for any top-level file with and extention
    //that "config" knows how to parse: yaml, json, etc.
    Config::builder()
        .add_source(config::File::with_name("configuration"))
        .add_source(config::Environment::with_prefix("R2P"))
        .build()?
        .try_deserialize()
}
