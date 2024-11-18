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
use tracing::{debug, error, info};
pub mod error;

#[derive(Clone)]
pub struct MessageBroker {
    pub connection: amqprs::connection::Connection,
}

pub struct ConsumerHandle {
    pub tag: String,
    pub rx: tokio::sync::mpsc::UnboundedReceiver<amqprs::channel::ConsumerMessage>,
    pub ch: amqprs::channel::Channel,
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
                info!("Connected to amqp: {}", config::CONFIG.ampq.host);
                conn
            }
            Err(err) => {
                error!("Error connecting to amqp: {}", err);
                panic!("Error connecting to amqp: {}", err);
            }
        };

        debug!("AMQP connection established successfully");

        Self {
            connection: amqp_conn,
        }
    }

    pub fn is_connected(&self) -> bool {
        debug!("Checking if AMQP connection is open");
        self.connection.is_open()
    }

    async fn create_channel(&self) -> Result<amqprs::channel::Channel> {
        debug!("Creating a new channel");
        let fut = self.connection.open_channel(None);

        let channel = match fut.await {
            Ok(channel) => {
                debug!("Channel created successfully");
                channel
            }
            Err(err) => {
                error!("Error creating channel: {}", err);
                return Err(Error::CreateChannelError(err));
            }
        };

        Ok(channel)
    }

    pub async fn telemetry_consumer(&self) -> Result<ConsumerHandle> {
        debug!("Starting telemetry consumer setup");
        let chan = self.create_channel().await?;

        let args = QueueDeclareArguments::default()
            .queue(CONFIG.ampq.telemetry_queue_name.clone())
            .finish();

        debug!("Declaring telemetry queue with args: {:?}", args);

        match chan.queue_declare(args).await {
            Ok(Some(v)) => {
                let (queue_name, _m, _) = v;
                debug!("Telemetry queue declared successfully: {}", queue_name);

                let args = BasicConsumeArguments::new(&queue_name, "");
                debug!("Starting basic consume with args: {:?}", args);

                let (consumer_tag, messages_rx) = chan
                    .basic_consume_rx(args)
                    .await
                    .map_err(|e| Error::ConsumerDeclareError(e))?;

                debug!(
                    "Telemetry consumer created successfully with tag: {}",
                    consumer_tag
                );

                Ok(ConsumerHandle {
                    tag: consumer_tag,
                    rx: messages_rx,
                    ch: chan,
                })
            }
            Ok(None) => {
                debug!("Failed to declare telemetry queue");
                Err(Error::FailedToDeclareQueue)
            }
            Err(err) => {
                debug!("Error declaring telemetry queue: {}", err);
                Err(Error::QueueDeclareError(err))
            }
        }
    }

    pub async fn registrar_consume(&self) -> Result<ConsumerHandle> {
        debug!("Starting registrar consumer setup");
        let chan = self.create_channel().await?;

        let args = QueueDeclareArguments::default()
            .queue(CONFIG.ampq.registrar_queue_name.clone())
            .finish();

        debug!("Declaring registrar queue with args: {:?}", args);

        match chan.queue_declare(args).await {
            Ok(Some(v)) => {
                let (queue_name, _m, _) = v;
                debug!("Registrar queue declared successfully: {}", queue_name);

                let args = BasicConsumeArguments::new(&queue_name, "");
                debug!("Starting basic consume with args: {:?}", args);

                let (consumer_tag, messages_rx) = chan
                    .basic_consume_rx(args)
                    .await
                    .map_err(|e| Error::ConsumerDeclareError(e))?;

                debug!(
                    "Registrar consumer created successfully with tag: {}",
                    consumer_tag
                );

                Ok(ConsumerHandle {
                    tag: consumer_tag,
                    rx: messages_rx,
                    ch: chan,
                })
            }
            Ok(None) => {
                debug!("Failed to declare registrar queue");
                Err(Error::FailedToDeclareQueue)
            }
            Err(err) => {
                debug!("Error declaring registrar queue: {}", err);
                Err(Error::QueueDeclareError(err))
            }
        }
    }

    pub async fn transmit_action(&self, payload: serde_json::Value) -> Result<serde_json::Value> {
        debug!("Starting action transmission with payload: {}", payload);

        let chan = self.create_channel().await?;

        let args = QueueDeclareArguments::default()
            .exclusive(true)
            .auto_delete(true)
            .finish();

        debug!("Declaring response queue with args: {:?}", args);

        let (response_queue_name, _m, _) = match chan.queue_declare(args).await {
            Ok(Some(v)) => {
                debug!("Response queue declared successfully: {}", v.0);
                v
            }
            Ok(None) => {
                debug!("Failed to declare response queue");
                return Err(Error::FailedToDeclareQueue);
            }
            Err(err) => {
                debug!("Error declaring response queue: {}", err);
                return Err(Error::QueueDeclareError(err));
            }
        };

        let correlation_id = uuid::Uuid::new_v4().to_string();
        debug!("Generated correlation ID: {}", correlation_id);

        let (tx, rx) = tokio::sync::oneshot::channel::<serde_json::Value>();
        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();

        let args = BasicConsumeArguments::new(&response_queue_name, "");
        debug!("Starting basic consume with args: {:?}", args);

        let (consumer_tag, mut messages_rx) = chan
            .basic_consume_rx(args)
            .await
            .map_err(|e| Error::ConsumerDeclareError(e))?;

        debug!("Consumer started with tag: {}", consumer_tag);

        tokio::spawn(async move {
            debug!("Spawned task to listen for messages");
            tokio::select! {
                _ = cancel_rx => {
                    debug!("Cancelled consumer task");
                    return
                }
                Some(msg) = messages_rx.recv() => {
                    if let Some(content) = msg.content {
                        match serde_json::from_slice::<serde_json::Value>(&content) {
                            Ok(json) => {
                                debug!("Message content deserialized successfully");
                                let _ = tx.send(json);
                            }
                            Err(err) => {
                                debug!("Failed to deserialize message content: {}", err);
                            }
                        }
                    }
                }
            }
        });

        let props = BasicProperties::default()
            .with_correlation_id(&correlation_id)
            .with_reply_to(&response_queue_name)
            .finish();

        debug!("Publishing message with properties: {:?}", props);

        let args = BasicPublishArguments::default()
            .routing_key(CONFIG.ampq.mqtt_gateway_queue_name.clone())
            .finish();

        let payload_bytes = serde_json::to_vec(&payload).map_err(Error::SerdeError)?;

        chan.basic_publish(props, payload_bytes, args)
            .await
            .map_err(|e| Error::PublishError(e))?;

        debug!("Message published, waiting for response");

        let response =
            match tokio::time::timeout(std::time::Duration::from_secs(CONFIG.ampq.timeout), rx)
                .await
            {
                Ok(v) => match v {
                    Ok(v) => {
                        debug!("Response received successfully");
                        v
                    }
                    Err(_) => {
                        debug!("Failed to receive response");
                        self.close_consumer(&chan, consumer_tag).await?;
                        return Err(Error::ResponseTimeout);
                    }
                },
                Err(_) => {
                    debug!("Response timed out");
                    self.close_consumer(&chan, consumer_tag).await?;
                    return Err(Error::ResponseTimeout);
                }
            };

        debug!("Cancelling response consumer");
        let _ = cancel_tx.send(());

        Ok(response)
    }

    async fn close_consumer<S>(
        &self,
        chan: &amqprs::channel::Channel,
        consumer_tag: S,
    ) -> Result<()>
    where
        S: Into<String>,
    {
        let args = BasicCancelArguments::default()
            .consumer_tag(consumer_tag.into())
            .finish();

        debug!("Closing consumer with args: {:?}", args);

        chan.basic_cancel(args)
            .await
            .map_err(|e| Error::ChannelError(e))?;

        debug!("Consumer closed successfully");
        Ok(())
    }
}
