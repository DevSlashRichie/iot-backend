use std::future::pending;

use clap::Parser;
use iot_db::SensorModel;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long)]
    pub host: Option<String>,
    #[arg(long)]
    pub broker: Option<String>,
    #[arg(long, env)]
    pub database_url: String,
}

pub fn read_args() -> CliArgs {
    dotenvy::dotenv().ok();
    CliArgs::parse()
}

#[tokio::main]
async fn main() {
    let args = read_args();
    pretty_env_logger::init();

    let db = match iot_db::connect_database(&args.database_url).await {
        Ok(db) => db,
        Err(err) => {
            println!("Could not connect to db: {}", err);
            return;
        }
    };

    let sensor_model: SensorModel = db.into();

    // Create a broadcast channel for sensor entries
    // We rely on type inference or explicit type if needed, but since both consumers expect Sender<SensorEntry>, it should infer.
    let (tx, _rx) = tokio::sync::broadcast::channel(100);

    // Spawn MQTT task
    if let Some(broker_host) = args.broker.clone() {
        let model = sensor_model.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let listener = iot_listener::IotListener::new(model, tx);
            if let Err(why) = listener.run_listener(broker_host).await {
                println!("MQTT failed: {}", why);
            }
        });
    }

    // Spawn HTTP task
    if let Some(host) = args.host.clone() {
        let model = sensor_model.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            if let Err(why) = iot_server::init_http_server(host, model, tx).await {
                println!("HTTP failed: {}", why);
            }
        });
    }

    // Keep the main alive forever.
    pending::<()>().await;
}