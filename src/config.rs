use config::{Config, ConfigError, File, FileFormat};
use once_cell::sync::Lazy;
use serde::Deserialize;

impl Default for ConfigStruct {
    fn default() -> Self {
        ConfigStruct {
            database: DatabaseConfig::default(),
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

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            protocol: "postgres".to_string(),
            user: "postgres".to_string(),
            password: "postgres".to_string(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: "".to_string(),
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

    pub fn db_url(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.database.protocol,
            self.database.user,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }
}

pub static CONFIG: Lazy<ConfigStruct> = Lazy::new(|| match ConfigStruct::new() {
    Ok(config) => config,
    Err(e) => panic!("Error loading config: {}", e),
});

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ConfigStruct {
    pub database: DatabaseConfig,
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

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub protocol: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub database: String,
}
