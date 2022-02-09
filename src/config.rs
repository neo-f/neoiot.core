use config::{Config, Environment, File};
use std::env;
use std::sync::RwLock;

lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::new());
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub postgres_url: String,
    pub secret: String,
}

impl Settings {
    pub fn new() -> Self {
        let mut s = Config::new();
        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}.yaml", env)).required(false))
            .unwrap();

        // Add in a local.yaml configuration file
        // This file shouldn't be checked in to git
        s.merge(File::with_name("config/local.yaml").required(false))
            .unwrap();

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("NEOIOT")).unwrap();
        s.try_into().unwrap()
    }
}
