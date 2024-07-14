// src/config.rs

use config::{ConfigError, Config, File, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub api_key: String,
    pub api_url: String,
    pub timeout_seconds: u64,
    pub environment: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::default();
        settings
            .merge(File::with_name("Settings"))? // Load settings from a file
            .merge(Environment::with_prefix("APP"))?; // Override with environment variables
        settings.try_into()
    }
}

