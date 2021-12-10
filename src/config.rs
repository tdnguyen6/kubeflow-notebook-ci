use serde::Deserialize;
pub use config::ConfigError;

#[derive(Deserialize, Default)]
pub struct Config {
    pub server: ServerCfg,
    pub kubeflow: KubeflowCfg,
    pub database_url: String,
    pub jwt_secret: String,
}

#[derive(Deserialize, Default)]
pub struct ServerCfg {
    pub host: String,
    pub port: i32,
}

#[derive(Deserialize, Default)]
pub struct KubeflowCfg {
    pub userid_header: String,
    pub interactive_endpoint: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        cfg.try_into()
    }
}
