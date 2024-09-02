use config::{Config, ConfigError, File, FileFormat};
use once_cell::sync::Lazy;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ConfigStruct {
    pub db: String,
    pub ampq: String,
    pub port: String,
    pub address: String,
    pub jwt: JwtConfig,
}

#[derive(Debug, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expires_in: u64,
    pub issuer: String,
}

impl Default for ConfigStruct {
    fn default() -> Self {
        ConfigStruct {
            db: "postgres://postgres:postgres@localhost:5432/iot-orchid".to_string(),
            ampq: "amqp://guest:guest@localhost:5672".to_string(),
            port: "3000".to_string(),
            address: "localhost".to_string(),
            jwt: JwtConfig::default(),
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        JwtConfig {
            secret: "secret".to_string(),
            expires_in: 60 * 60 * 5,
            issuer: "localhost".to_string(),
        }
    }
}

impl ConfigStruct {
    fn new() -> Result<Self, ConfigError> {
        let builder =
            Config::builder().add_source(File::new("config/settings_dev", FileFormat::Yaml));

        let config = builder.build()?;

        Ok(config.try_deserialize()?)
    }
}

pub static CONFIG: Lazy<ConfigStruct> = Lazy::new(|| match ConfigStruct::new() {
    Ok(config) => config,
    Err(e) => panic!("Error loading config: {}", e),
});
