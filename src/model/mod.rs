use super::config;
use futures::executor::block_on;
pub mod cluster;
mod common;
pub mod error;
pub mod microdevice;
mod msg_broker;
#[allow(unused_imports)]
use error::{Error, Result};

#[derive(Clone)]
pub struct ModelManager {
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) msg_broker: msg_broker::MessageBroker,
}

impl ModelManager {
    pub async fn new() -> Self {
        let fut = sea_orm::Database::connect(config::CONFIG.db_url());

        let sea_orm_db = match block_on(fut) {
            Ok(db) => {
                println!("Connected to database");
                db
            }
            Err(err) => {
                panic!("Error connecting to database: {}", err);
            }
        };

        let msg_broker = msg_broker::MessageBroker::new().await;

        Self {
            db: sea_orm_db,
            msg_broker,
        }
    }
}
