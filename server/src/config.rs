pub use config::ConfigError;
use serde::Deserialize;

#[derive(Deserialize, Default, Clone)]
pub struct Config {
    pub server: ServerCfg,
    pub database_url: String,
    pub database_maxcon: u32,
    pub cluster_container_registry: String,
    pub default_image_name: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct ServerCfg {
    pub host: String,
    pub port: i32,
    pub base_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        cfg.try_into()
    }
}
