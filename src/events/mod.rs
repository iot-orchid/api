use amqprs::{
    channel::{BasicAckArguments, BasicNackArguments, BasicPublishArguments, ConsumerMessage},
    BasicProperties,
};
use serde::{Deserialize, Serialize};
use tokio::select;
use tracing::{debug, error, info};

use crate::{
    context::Ctx,
    model::{microdevice::MicrodeviceBaseModelController as MicrodeviceBMC, ModelManager},
};

pub struct EventManager {
    model_manager: ModelManager,
}

#[derive(Deserialize, Debug)]
struct RegistrarMessage {
    device_id: String,
}

#[derive(Serialize, Debug)]
struct RegistrarResponse {
    device_id: String,
    cluster_id: String,
    topics: Vec<String>,
    name: String,
}

impl EventManager {
    pub fn new(mm: ModelManager) -> Self {
        EventManager { model_manager: mm }
    }

    pub async fn handle_registration(
        &self,
        ch: &amqprs::channel::Channel,
        msg: Option<ConsumerMessage>,
    ) {
        // Early return if no message is received
        let msg = match msg {
            Some(m) => m,
            None => {
                error!("No message received for registration handling.");
                return;
            }
        };

        // Extract message content, return early on failure
        let content = match msg.content.as_deref() {
            Some(content) => content,
            None => {
                error!("Message content is missing.");
                return;
            }
        };

        debug!("Payload: {:}", std::str::from_utf8(content).unwrap());

        // Deserialize the registration message
        let registrar_msg: RegistrarMessage = match serde_json::from_slice(content) {
            Ok(parsed) => parsed,
            Err(e) => {
                error!(error = %e, "Failed to deserialize registration message payload.");
                if let Some(deliver) = &msg.deliver {
                    let nack_args = BasicNackArguments::new(deliver.delivery_tag(), false, false);
                    if let Err(e) = ch.basic_nack(nack_args).await {
                        error!(error = %e, "Failed to nack message.");
                    } else {
                        info!("Message nacked and requeued successfully.");
                    }
                }
                return;
            }
        };

        // Construct the context
        let ctx = Ctx::MicrodeviceCtx {
            device_id: registrar_msg.device_id,
            cluster_id: "idk".to_string(), // Placeholder
        };

        // Retrieve the microdevice record
        let record = match MicrodeviceBMC::get_microdevice(&ctx, &self.model_manager).await {
            Ok(rec) => rec,
            Err(e) => {
                error!(error = %e, "Failed to retrieve microdevice record.");
                return;
            }
        };

        // Process and respond if `reply_to` is set in the message properties
        if let Some(props) = &msg.basic_properties {
            if let Some(reply_to) = props.reply_to() {
                let payload = match serde_json::to_vec(&record) {
                    Ok(data) => data,
                    Err(e) => {
                        error!(error = %e, "Failed to serialize response payload.");
                        return;
                    }
                };

                let publish_args = BasicPublishArguments::new("", reply_to).finish();
                if let Err(e) = ch.basic_publish(props.clone(), payload, publish_args).await {
                    error!(error = %e, "Failed to publish response message.");
                    return;
                }

                info!(reply_to = %reply_to, "Successfully published response message.");
            }
        }

        // Acknowledge the message delivery if `deliver` is present
        if let Some(deliver) = &msg.deliver {
            let ack_args = BasicAckArguments::new(deliver.delivery_tag(), false);
            if let Err(e) = ch.basic_ack(ack_args).await {
                error!(error = %e, "Failed to acknowledge message.");
            } else {
                info!("Message acknowledged successfully.");
            }
        }
    }

    pub async fn start(&self) {
        info!("Starting event manager");

        loop {
            if self.model_manager.ampq_bridge.is_connected() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        let mut telemetry_consumer = match self.model_manager.ampq_bridge.telemetry_consumer().await
        {
            Ok(v) => v,
            Err(err) => {
                info!("Error starting telemetry consumer: {}", err);
                return;
            }
        };

        info!("Telemetry consumer started");

        let mut registrar_consumer = match self.model_manager.ampq_bridge.registrar_consume().await
        {
            Ok(v) => v,
            Err(err) => {
                info!("Error starting registrar consumer: {}", err);
                return;
            }
        };

        info!("Registrar consumer started");

        loop {
            select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(60 * 2)) => {
                    info!("Event manager running");
                }

                msg = telemetry_consumer.rx.recv() => {
                    // self.handle_registration(&ch, msg).await
                }

                msg = registrar_consumer.rx.recv() => {
                    self.handle_registration(&registrar_consumer.ch, msg).await;
                }
            }
        }
    }
}
