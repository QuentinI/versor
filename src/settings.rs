use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Telegram {
    pub token: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Constants {
    pub divisor: u32,
    pub cache_cycle: u8,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub options: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub telegram: Telegram,
    pub constants: Constants,
    pub database: Database,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.merge(File::with_name("config/default"))?;
        s.merge(File::with_name("config/develop").required(false))?;
        s.merge(Environment::with_prefix("APP").separator("_"))?;

        s.try_into()
    }
}
