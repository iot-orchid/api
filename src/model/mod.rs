use super::config;
use futures::executor::block_on;
mod ampq;
pub mod cluster;
mod common;
pub mod error;
pub mod microdevice;
#[allow(unused_imports)]
use error::{Error, Result};
use tracing::{info, debug, error};

#[derive(Clone)]
pub struct ModelManager {
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) ampq_bridge: ampq::MessageBroker,
}

impl ModelManager {
    pub async fn new() -> Self {
        let fut = sea_orm::Database::connect(config::CONFIG.db_url());

        let sea_orm_db = match block_on(fut) {
            Ok(db) => {
                info!("Connected to database: {}", config::CONFIG.db_url());
                db
            }
            Err(err) => {
                error!("Error connecting to database: {}", err);
                panic!("Error connecting to database: {}", err);
            }
        };

        let msg_broker = ampq::MessageBroker::new().await;

        Self {
            db: sea_orm_db,
            ampq_bridge: msg_broker,
        }
    }
}
