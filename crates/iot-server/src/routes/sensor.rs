use axum::{
    extract::{Path, State},
    response::Json,
    routing::get,
    Router,
};
use iot_entities::{Sensor, SensorEntry, SensorService};
use uuid::Uuid;

async fn get_all_sensors<S>(State(state): State<S>) -> Json<Vec<Sensor>>
where
    S: SensorService + Clone + Sync + Send + 'static,
{
    let sensors = state.fetch_all().await.unwrap();

    Json(sensors)
}

async fn get_sensor<S>(Path(id): Path<Uuid>, State(state): State<S>) -> Json<Sensor>
where
    S: SensorService + Clone + Sync + Send + 'static,
{
    let sensor = state.fetch_one(id).await.unwrap();
    Json(sensor)
}

async fn get_sensor_history<S>(
    Path(id): Path<Uuid>,
    State(state): State<S>,
) -> Json<Vec<SensorEntry>>
where
    S: SensorService + Clone + Sync + Send + 'static,
{
    let history = state.fetch_history(id).await.unwrap();
    Json(history)
}

pub fn sensor_routes<S>() -> Router<S>
where
    S: SensorService + Send + Sync + Clone + 'static,
{
    Router::<S>::new()
        .route("/", get(get_all_sensors::<S>))
        .route("/{id}", get(get_sensor::<S>))
        .route("/{id}/history", get(get_sensor_history::<S>))
}
