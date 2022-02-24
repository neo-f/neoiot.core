use config::{Config, ConfigError, Environment, File};
use std::env;
use std::sync::RwLock;

lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::default().unwrap());
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub postgres_url: String,
    pub secret: String,
}

impl Settings {
    pub fn default() -> Result<Self, ConfigError> {
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let builder = Config::builder()
            .add_source(File::with_name(&format!("config/{}.yaml", env)).required(false))
            .add_source(File::with_name("config/local.yaml").required(false))
            .add_source(Environment::with_prefix("NEOIOT"));
        builder.build()?.try_deserialize()
    }
}
