use super::config;
use crate::config::CONFIG;
use amqprs::{
    channel::{
        BasicCancelArguments, BasicConsumeArguments, BasicPublishArguments, QueueDeclareArguments,
    },
    consumer::{self, DefaultConsumer},
    BasicProperties,
};
#[allow(unused_imports)]
use error::{Error, Result};
pub mod error;

#[derive(Clone)]
pub struct MessageBroker {
    #[allow(dead_code)]
    pub _connection: amqprs::connection::Connection,
    pub channel: amqprs::channel::Channel,
}

impl MessageBroker {
    pub async fn new() -> Self {
        let args = amqprs::connection::OpenConnectionArguments::new(
            &config::CONFIG.ampq.host,
            config::CONFIG.ampq.port,
            &config::CONFIG.ampq.user,
            &config::CONFIG.ampq.password,
        )
        .finish();

        let fut = amqprs::connection::Connection::open(&args);

        let amqp_conn = match fut.await {
            Ok(conn) => {
                println!("Connected to amqp");
                conn
            }
            Err(err) => {
                panic!("Error connecting to amqp: {}", err);
            }
        };

        let fut = amqp_conn.open_channel(None);

        let channel = match fut.await {
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

        let queue = match fut.await {
            Ok(v) => {
                println!("Queue declared: {:?}", v);
                v
            }
            Err(err) => {
                match channel.close().await {
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
            _connection: amqp_conn,
            channel,
        }
    }

    pub async fn transmit_action(&self, payload: serde_json::Value) -> Result<serde_json::Value> {
        // let args = QueueDeclareArguments::default()
        //     .exclusive(true)
        //     .auto_delete(true)
        //     .finish();

        // let (response_queue_name, _m, _) = match self.channel.queue_declare(args).await {
        //     Ok(Some(v)) => v,
        //     Ok(None) => return Err(Error::FailedToDeclareQueue),
        //     Err(err) => return Err(Error::QueueDeclareError(err)),
        // };

        // // Unique correlation id and response receiver
        // let correlation_id = uuid::Uuid::new_v4().to_string();
        // let (tx, rx) = tokio::sync::oneshot::channel::<serde_json::Value>();
        // let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();

        // let args = BasicConsumeArguments::new(&response_queue_name, "");

        // let (consumer_tag, mut messages_rx) = self
        //     .channel
        //     .basic_consume_rx(args)
        //     .await
        //     .map_err(|e| Error::ChannelError(e))?;

        // // Spawn a task to listen for messages and filter based on correlation ID
        // tokio::spawn(async move {
        //     tokio::select! {
        //         _ = cancel_rx => {
        //             // Cancelled by the transmitter, exit the task
        //             println!("Response listener cancelled.");
        //             return;
        //         }
        //         Some(message) = messages_rx.recv() => {
        //             todo!()
        //         }
        //     }
        // });

        let props = BasicProperties::default()
            // .with_correlation_id(&correlation_id)
            // .with_reply_to(&response_queue_name)
            .finish();

        let args = BasicPublishArguments::default()
            .routing_key(CONFIG.ampq.mqtt_gateway_queue_name.clone())
            .finish();

        let payload_bytes = serde_json::to_vec(&payload).map_err(|e| Error::SerdeError(e))?;

        self.channel
            .basic_publish(props, payload_bytes, args)
            .await
            .unwrap();

        // let response =
        //     match tokio::time::timeout(std::time::Duration::from_secs(CONFIG.ampq.timeout), rx)
        //         .await
        //     {
        //         Ok(v) => match v {
        //             Ok(v) => v,
        //             Err(_) => {
        //                 self.close_consumer(consumer_tag).await?;
        //                 return Err(Error::ResponseTimeout);
        //             }
        //         },
        //         Err(_) => {
        //             self.close_consumer(consumer_tag).await?;
        //             return Err(Error::ResponseTimeout);
        //         }
        //     };

        // cancel_tx.send(());

        Ok(payload)
    }

    async fn close_consumer<S>(&self, consumer_tag: S) -> Result<()>
    where
        S: Into<String>,
    {
        let args = BasicCancelArguments::default()
            .consumer_tag(consumer_tag.into())
            .finish();

        self.channel
            .basic_cancel(args)
            .await
            .map_err(|e| Error::ChannelError(e))?;

        Ok(())
    }
}
