use tokio::select;
use tracing::info;

use crate::model::ModelManager;

pub struct EventManager {
    model_manager: ModelManager,
}

impl EventManager {
    pub fn new(mm: ModelManager) -> Self {
        EventManager { model_manager: mm }
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
                    match msg {
                        Some(msg) => {
                            info!("Received a telemetry message: {:?}", msg.content);

                            
                        }
                        None => {
                            info!("Channel closed");
                            break;
                        }
                    }
                }

                msg = registrar_consumer.rx.recv() => {
                    match msg {
                        Some(msg) => {
                            info!("Received a registrar message: {:?}", msg.content);
                        }
                        None => {
                            info!("Channel closed");
                            break;
                        }
                    }
                }
            }
        }
    }
}
