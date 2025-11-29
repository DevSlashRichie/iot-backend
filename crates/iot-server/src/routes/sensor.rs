use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::{sse::Event, IntoResponse, Json, Sse},
    routing::get,
    Error, Router,
};
use iot_entities::{Sensor, SensorEntry, SensorService};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use uuid::Uuid;

use crate::AppState;

async fn get_all_sensors<S>(State(state): State<AppState<S>>) -> Json<Vec<Sensor>>
where
    S: SensorService + Clone + Sync + Send + 'static,
{
    let sensors = state.service.fetch_all().await.unwrap();

    Json(sensors)
}

async fn get_sensor<S>(Path(id): Path<Uuid>, State(state): State<AppState<S>>) -> Json<Sensor>
where
    S: SensorService + Clone + Sync + Send + 'static,
{
    let sensor = state.service.fetch_one(id).await.unwrap();
    Json(sensor)
}

async fn get_sensor_history<S>(
    Path(id): Path<Uuid>,
    State(state): State<AppState<S>>,
) -> Json<Vec<SensorEntry>>
where
    S: SensorService + Clone + Sync + Send + 'static,
{
    let history = state.service.fetch_history(id).await.unwrap();
    Json(history)
}

async fn sse_handler<S>(
    Path(id): Path<Uuid>,
    State(state): State<AppState<S>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Error>>>
where
    S: SensorService + Clone + Sync + Send + 'static,
{
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |result| match result {
        Ok(entry) => {
            if entry.sensor_id == id {
                Some(Event::default().json_data(entry).map_err(Error::new))
            } else {
                None
            }
        }
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

async fn ws_handler<S>(
    ws: WebSocketUpgrade,
    Path(id): Path<Uuid>,
    State(state): State<AppState<S>>,
) -> impl IntoResponse
where
    S: SensorService + Clone + Sync + Send + 'static,
{
    ws.on_upgrade(move |socket| handle_socket(socket, id, state))
}

async fn handle_socket<S>(mut socket: WebSocket, id: Uuid, state: AppState<S>) {
    let mut rx = state.tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(msg) => {
                if msg.sensor_id == id {
                    let payload = match serde_json::to_string(&msg) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };
                    if socket.send(Message::Text(payload.into())).await.is_err() {
                        break;
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
}

pub fn sensor_routes<S>() -> Router<AppState<S>>
where
    S: SensorService + Send + Sync + Clone + 'static,
{
    Router::new()
        .route("/", get(get_all_sensors::<S>))
        .route("/{id}", get(get_sensor::<S>))
        .route("/{id}/history", get(get_sensor_history::<S>))
        .route("/{id}/live", get(sse_handler::<S>))
        .route("/{id}/ws", get(ws_handler::<S>))
}
