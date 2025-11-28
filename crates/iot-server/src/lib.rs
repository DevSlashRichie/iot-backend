mod errors;
mod routes;

use axum::Router;
use iot_entities::SensorService;
use log::info;
use tokio::net::TcpListener;
use tower_http::cors::{self, CorsLayer};

pub async fn init_http_server<T, S>(host: T, service: S) -> Result<(), errors::ServerError>
where
    T: Into<String>,
    S: SensorService + Clone + Send + Sync + 'static,
{
    // build our application with a single route
    let app = Router::<S>::new()
        .nest("/sensor", routes::sensor::sensor_routes())
        .layer(
            CorsLayer::new()
                .allow_methods(cors::Any)
                .allow_origin(cors::Any),
        )
        .with_state(service);

    // run our app with hyper, listening globally on port 3000
    let host = host.into();
    let listener = TcpListener::bind(host).await?;
    info!("Listening on {}", listener.local_addr().unwrap());
    _ = axum::serve(listener, app).await?;

    Ok(())
}
