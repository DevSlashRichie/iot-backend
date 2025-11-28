use axum::{routing::post, Router};
use iot_entities::SensorService;

use crate::AppState;

async fn ping_route<S>() -> &'static str
where
    S: SensorService + Clone + Sync + Send + 'static,
{
    "pong"
}

pub fn ping_routes<S>() -> Router<AppState<S>>
where
    S: SensorService + Send + Sync + Clone + 'static,
{
    Router::new().route("/", post(ping_route::<S>))
}