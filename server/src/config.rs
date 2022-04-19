pub use config::ConfigError;
use serde::Deserialize;

#[derive(Deserialize, Default, Clone)]
pub struct Config {
    pub server: ServerCfg,
    pub database_url: String,
    pub database_maxcon: u32,
    pub kubeflow: KubeflowCfg,
    pub default_image_name: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct ServerCfg {
    pub host: String,
    pub port: i32,
    pub base_path: String,
}


#[derive(Deserialize, Default, Clone)]
pub struct KubeflowCfg {
    pub userid_header: String,
    pub notebook_resource: String,
    pub profile_resource: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        cfg.try_into()
    }
}
