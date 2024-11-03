use super::config;
use amqprs::channel::{Channel, QueueDeclareArguments};
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
    pub(crate) amqp: amqprs::connection::Connection,
    pub(crate) channel : amqprs::channel::Channel,
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

        let fut = amqp_conn.open_channel(None);

        let channel = match block_on(fut) {
            Ok(channel) => {
                println!("Opened channel");
                channel
            }
            Err(err) => {
                panic!("Error opening channel: {}", err);
            }
        };

        let fut = channel.queue_declare(
            QueueDeclareArguments::new(config::CONFIG.ampq.mqtt_gateway_queue_name.as_str())
                .finish(),
        );

        let queue = match block_on(fut) {
            Ok(v) => {
                println!("Queue declared: {:?}", v);
                v
            }
            Err(err) => {
                match block_on(channel.close()) {
                    Ok(_) => {
                        eprintln!("Error: Failed to declare worker queue: {}. Channel closed successfully.", err);
                        std::process::exit(1);
                    }
                    Err(close_err) => {
                        eprintln!("Error: Failed to declare worker queue: {}. Additionally, failed to close channel: {}", err, close_err);
                        std::process::exit(1);
                    }
                }
            }
        };

        Self {
            db: sea_orm_db,
            amqp: amqp_conn,
            channel : channel
        }
    }
}
