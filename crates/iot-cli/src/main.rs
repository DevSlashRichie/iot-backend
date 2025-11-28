use clap::Parser;
use iot_db::SensorModel;
use iot_listener::IotListener;
use tokio::try_join;

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

    let mqtt_future = async {
        if let Some(broker_host) = args.broker.clone() {
            let listener = IotListener::from(sensor_model.clone());
            listener.run_listener(broker_host).await?;
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    let http_future = async {
        if let Some(host) = args.host.clone() {
            iot_server::init_http_server(host, sensor_model.clone()).await?;
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    // Run in parallel
    if let Err(e) = try_join!(mqtt_future, http_future) {
        eprintln!("Task failed: {}", e);
    }
}
