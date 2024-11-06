pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum Error {
    QueueDeclareError(amqprs::error::Error),
    ConnectionError(amqprs::error::Error),
    ChannelError(amqprs::error::Error),
    CloseConsumerError(amqprs::error::Error),
    SerdeError(serde_json::Error),
    CommunicationError(std::sync::mpsc::RecvError),
    ResponseTimeout,
    FailedToDeclareQueue,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::QueueDeclareError(e) => write!(f, "Queue declare error: {}", e),
            Error::ConnectionError(e) => write!(f, "Connection error: {}", e),
            Error::ChannelError(e) => write!(f, "Channel error: {}", e),
            Error::SerdeError(e) => write!(f, "Serde error: {}", e),
            Error::ResponseTimeout => write!(f, "AMQP broker failed to respond in time"),
            Error::CommunicationError(e) => write!(f, "Communication error: {}", e),
            Error::CloseConsumerError(e) => write!(f, "Close consumer error: {}", e),
            Error::FailedToDeclareQueue => write!(f, "Failed to declare queue"),
        }
    }
}
