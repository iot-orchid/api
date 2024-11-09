use super::config;
use crate::config::CONFIG;
use amqprs::{
    channel::{
        BasicCancelArguments, BasicConsumeArguments, BasicPublishArguments, QueueDeclareArguments,
    },
    BasicProperties,
};
#[allow(unused_imports)]
use error::{Error, Result};
use tracing::info;
pub mod error;

#[derive(Clone)]
pub struct MessageBroker {
    pub connection: amqprs::connection::Connection,
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

        Self {
            connection: amqp_conn,
        }
    }

    async fn create_channel(&self) -> Result<amqprs::channel::Channel> {
        let fut = self.connection.open_channel(None);

        let channel = match fut.await {
            Ok(channel) => {
                channel
            }
            Err(err) => {
                return Err(Error::CreateChannelError(err));
            }
        };

        Ok(channel)
    }

    async fn declare_mqtt_gateway_queue(&self, chan: &amqprs::channel::Channel) -> Result<()> {
        let args = QueueDeclareArguments::default()
            .queue(config::CONFIG.ampq.mqtt_gateway_queue_name.clone())
            .durable(true)
            .finish();

        chan
            .queue_declare(args)
            .await
            .map_err(|e| Error::QueueDeclareError(e))?;

        Ok(())
    }

    pub async fn transmit_action(&self, payload: serde_json::Value) -> Result<serde_json::Value> {

        let chan = self.create_channel().await?;

        let args = QueueDeclareArguments::default()
            .exclusive(true)
            .auto_delete(true)
            .finish();

        let (response_queue_name, _m, _) = match chan.queue_declare(args).await {
            Ok(Some(v)) => v,
            Ok(None) => return Err(Error::FailedToDeclareQueue),
            Err(err) => return Err(Error::QueueDeclareError(err)),
        };

        // Unique correlation id and response receiver
        let correlation_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel::<serde_json::Value>();
        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();

        let args = BasicConsumeArguments::new(&response_queue_name, "");

        let (consumer_tag, mut messages_rx) = chan
            .basic_consume_rx(args)
            .await
            .map_err(|e| Error::ConsumerDeclareError(e))?;

        // Spawn a task to listen for messages and filter based on correlation ID
        tokio::spawn(async move {
            tokio::select! {
                _ = cancel_rx => {
                    // Cancelled by the transmitter, exit the task
                    info!("Cancelled consumer task");
                    return
                }
                Some(msg) = messages_rx.recv() => {
                    
                    if let Some(content) = msg.content {
                        match serde_json::from_slice::<serde_json::Value>(&content) {
                            Ok(json) => {
                                let _ = tx.send(json);
                            }
                            Err(err) => {
                                eprintln!("Failed to deserialize message content: {}", err);
                            }
                        }
                    }

                    return
                }
            }
        });

        // Causes a channel to be closed when the future is dropped
        // self.declare_mqtt_gateway_queue(&chan).await?;

        let props = BasicProperties::default()
            .with_correlation_id(&correlation_id)
            .with_reply_to(&response_queue_name)
            .finish();

        let args = BasicPublishArguments::default()
            .routing_key(CONFIG.ampq.mqtt_gateway_queue_name.clone())
            .finish();

        let payload_bytes = serde_json::to_vec(&payload).map_err(Error::SerdeError)?;

        chan
            .basic_publish(props, payload_bytes, args)
            .await
            .map_err(|e| Error::PublishError(e))?;

        let response =
            match tokio::time::timeout(std::time::Duration::from_secs(CONFIG.ampq.timeout), rx)
                .await
            {
                Ok(v) => match v {
                    Ok(v) => v,
                    Err(e) => {
                        self.close_consumer(&chan, consumer_tag).await?;
                        return Err(Error::ResponseTimeout);
                    }
                },
                Err(_) => {
                    self.close_consumer(&chan, consumer_tag).await?;
                    return Err(Error::ResponseTimeout);
                }
            };

        cancel_tx.send(());

        Ok(response)
    }

    async fn close_consumer<S>(&self, chan: &amqprs::channel::Channel, consumer_tag: S) -> Result<()>
    where
        S: Into<String>,
    {
        let args = BasicCancelArguments::default()
            .consumer_tag(consumer_tag.into())
            .finish();

        chan
            .basic_cancel(args)
            .await
            .map_err(|e| Error::ChannelError(e))?;

        Ok(())
    }
}
