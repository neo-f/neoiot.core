use config::{Config, ConfigError, Environment, File};
use std::{env, sync::Arc};

lazy_static! {
    pub static ref SETTINGS: Arc<Settings> = Arc::new(Settings::default().unwrap());
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub core: CoreConfig,
    pub emqx: EmqxConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CoreConfig {
    pub endpoint: String,
    pub postgres_dsn: String,
    pub redis_dsn: String,
    pub secret: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EmqxConfig {
    pub management_host: String,
    pub app_id: String,
    pub app_secret: String,
}

impl Settings {
    pub fn default() -> Result<Self, ConfigError> {
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let builder = Config::builder()
            .add_source(File::with_name("config/local.yaml").required(false))
            .add_source(File::with_name(&format!("config/{}.toml", env)).required(false))
            .add_source(File::with_name("./config.toml").required(false))
            .add_source(File::with_name("~/.config/neoiot/config.toml").required(false))
            .add_source(File::with_name("/etc/neoiot/config.toml").required(false))
            .add_source(Environment::with_prefix("NEOIOT"))
            .set_default("endpoint", "0.0.0.0:3000")?;
        builder.build()?.try_deserialize()
    }
}
