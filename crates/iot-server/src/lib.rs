mod errors;
pub mod routes;

use axum::Router;
use iot_entities::{SensorEntry, SensorService};
use log::info;
use tokio::{net::TcpListener, sync::broadcast};
use tower_http::cors::{self, CorsLayer};

#[derive(Clone)]
pub struct AppState<S> {
    pub service: S,
    pub tx: broadcast::Sender<SensorEntry>,
}

pub async fn init_http_server<T, S>(
    host: T,
    service: S,
    tx: broadcast::Sender<SensorEntry>,
) -> Result<(), errors::ServerError>
where
    T: Into<String>,
    S: SensorService + Clone + Send + Sync + 'static,
{
    let state = AppState { service, tx };

    // build our application with a single route
    let app = Router::new()
        .nest("/ping", routes::ping::ping_routes())
        .nest("/sensor", routes::sensor::sensor_routes())
        .layer(
            CorsLayer::new()
                .allow_methods(cors::Any)
                .allow_origin(cors::Any),
        )
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let host = host.into();
    let listener = TcpListener::bind(host).await?;
    info!("Listening on {}", listener.local_addr().unwrap());
    _ = axum::serve(listener, app).await?;

    Ok(())
}
