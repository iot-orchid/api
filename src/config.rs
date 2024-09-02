use config::{Config, File, FileFormat};
use once_cell::sync::Lazy;
use std::collections::HashMap;
#[derive(Debug)]
pub struct ConfigStruct {
    pub db: String,
    pub _ampq: String,
    pub port: String,
    pub address: String,
    pub jwt_secret: String,
}

impl Default for ConfigStruct {
    fn default() -> Self {
        ConfigStruct {
            db: "postgres://postgres:postgres@localhost:5432/iot-orchid".to_string(),
            _ampq: "amqp://guest:guest@localhost:5672".to_string(),
            port: "3000".to_string(),
            address: "localhost".to_string(),
            jwt_secret: "secret".to_string(),
        }
    }
}

impl ConfigStruct {
    fn new() -> Self {
        let builder =
            Config::builder().add_source(File::new("config/settings_dev", FileFormat::Yaml));

        match builder.build() {
            Ok(config) => match config.cache.into_table().as_ref() {
                Ok(tbl) => ConfigStruct {
                    db: get_value("db", tbl).to_string(),
                    _ampq: get_value("ampq", tbl).to_string(),
                    port: get_value("port", tbl).to_string(),
                    address: get_value("address", tbl).to_string(),
                    jwt_secret: get_value("jwt_secret", tbl).to_string(),
                },
                Err(err) => {
                    eprintln!("Error: {:?}", err);

                    ConfigStruct {
                        ..Default::default()
                    }
                }
            },
            Err(err) => {
                eprintln!("Error: {:?}", err);
                ConfigStruct {
                    ..Default::default()
                }
            }
        }
    }
}

pub static CONFIG: Lazy<ConfigStruct> = Lazy::new(|| ConfigStruct::new());

fn get_value<'a>(key: &'static str, map: &'a HashMap<String, config::Value>) -> &'a config::Value {
    match map.get(key) {
        Some(val) => val,
        None => panic!("{key} is not defined in the config YAML"),
    }
}
