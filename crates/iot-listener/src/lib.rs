mod errors;

use std::str::FromStr;

use log::{error, info};
use mqtt_async_client::{
    client::{ClientBuilder, QoS, Subscribe, SubscribeTopic},
    Error,
};
use tokio::sync::broadcast::Sender;

use iot_entities::{SensorEntry, SensorService};
use uuid::Uuid;

#[derive(Clone)]
pub struct IotListener<S>
where
    S: SensorService,
{
    svc: S,
    tx: Sender<SensorEntry>,
}

impl<S: SensorService + Send + Sync + 'static> IotListener<S> {
    pub fn new(svc: S, tx: Sender<SensorEntry>) -> Self {
        Self { svc, tx }
    }

    async fn save_entry(&self, id: String, data: String) {
        let uuid = match Uuid::from_str(&id) {
            Ok(ok) => ok,
            Err(err) => {
                error!("Could not parse uuid: {}", err);
                return;
            }
        };

        let ppm = match f64::from_str(&data) {
            Ok(ok) => ok,
            Err(err) => {
                error!("Could not parse ppm: {}", err);
                return;
            }
        };

        match self.svc.save_entry(uuid, ppm).await {
            Ok(entry) => {
                if let Err(err) = self.tx.send(entry) {
                    error!("Could not broadcast entry: {}", err);
                }
            }
            Err(why) => {
                error!("could not save entry record {}", why);
            }
        }
    }

    async fn register_sensor(&self, id: String, label: String) {
        let uuid = match Uuid::from_str(&id) {
            Ok(ok) => ok,
            Err(err) => {
                error!("Could not parse uuid: {}", err);
                return;
            }
        };

        if let Err(why) = self.svc.register_sensor(uuid, label).await {
            error!("could not save sensor record {}", why);
        }
    }

    pub async fn run_listener(&self, broker_host: String) -> Result<(), errors::ListenerError> {
        info!("Subscriber at iot/sensor/gas {} inited.", &broker_host);
        let mut client = ClientBuilder::default()
            .set_url_string(&broker_host)?
            .build()?;

        client.connect().await?;

        let register_gas_sensor = SubscribeTopic {
            topic_path: "iot/sensor/register".to_string(),
            qos: QoS::AtLeastOnce,
        };

        let read_gas_entry_topic = SubscribeTopic {
            topic_path: "iot/sensor/gas".to_string(),
            qos: QoS::AtMostOnce,
        };

        let sub = Subscribe::new(vec![read_gas_entry_topic, register_gas_sensor]);

        let subres = client.subscribe(sub).await?;
        subres.any_failures()?;

        loop {
            let r = client.read_subscriptions().await;

            match r {
                Err(Error::Disconnected) => break,
                Ok(r) => {
                    let payload = r.payload();
                    let data = String::from_utf8_lossy(payload);
                    let data = data.to_string();
                    let topic = r.topic();

                    if topic == "iot/sensor/gas" {
                        if let Some((id, ppm)) = data.split_once("|") {
                            info!("(iot/sensor/gas) {}: {}", &id, &data);
                            let id = id.to_owned();
                            let ppm = ppm.to_owned();
                            let s = self.clone();

                            tokio::spawn(async move {
                                Self::save_entry(&s, id, ppm).await;
                            });
                        }
                    } else if topic == "iot/sensor/register" {
                        if let Some((id, label)) = data.split_once("|") {
                            info!("(iot/sensor/register) {}: {}", &id, &data);
                            let id = id.to_owned();
                            let label = label.to_owned();
                            let s = self.clone();

                            tokio::spawn(async move {
                                Self::register_sensor(&s, id, label).await;
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}