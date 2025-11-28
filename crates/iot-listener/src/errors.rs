use thiserror::Error;

#[derive(Error, Debug)]
pub enum ListenerError {
    #[error("got mqtt error: {0}")]
    Mqtt(#[from] mqtt_async_client::Error),
}
