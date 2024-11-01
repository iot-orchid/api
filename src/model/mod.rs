use super::config;
use futures::executor::block_on;
pub mod cluster;
mod common;
pub mod error;
pub mod microdevice;
#[allow(unused_imports)]
use error::{Error, Result};

#[derive(Clone)]
pub struct ModelManager {
    pub(crate) db: sea_orm::DatabaseConnection,
    pub(crate) _amqp: amqprs::connection::Connection,
    
}

impl ModelManager {
    pub fn new() -> Self {
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

        let args = amqprs::connection::OpenConnectionArguments::new(
            &config::CONFIG.ampq.host,
            config::CONFIG.ampq.port,
            &config::CONFIG.ampq.user,
            &config::CONFIG.ampq.password,
        )
        .finish();

        let fut = amqprs::connection::Connection::open(&args);

        let amqp_conn = match block_on(fut) {
            Ok(conn) => {
                println!("Connected to amqp");
                conn
            }
            Err(err) => {
                panic!("Error connecting to amqp: {}", err);
            }
        };

        Self {
            db: sea_orm_db,
            _amqp: amqp_conn,
        }
    }
}
