use super::config;
use amqprs::channel::QueueDeclareArguments;

#[derive(Clone)]
pub struct MessageBroker {
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
}
