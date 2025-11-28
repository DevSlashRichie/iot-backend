use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
pub struct Sensor {
    pub id: Uuid,
    pub label: String,
    pub created_at: i64,
}

#[derive(Serialize, sqlx::FromRow, Debug)]
pub struct SensorEntry {
    pub id: Uuid,
    pub sensor_id: Uuid,
    pub value: f64,
    pub created_at: i64,
}

pub trait SensorService: Clone {
    type Error: std::error::Error;

    fn save_entry(
        &self,
        sensor_id: Uuid,
        ppm: f64,
    ) -> impl Future<Output = Result<SensorEntry, Self::Error>> + Send;

    fn register_sensor(
        &self,
        id: Uuid,
        label: String,
    ) -> impl Future<Output = Result<Sensor, Self::Error>> + Send;

    fn fetch_one(&self, id: Uuid) -> impl Future<Output = Result<Sensor, Self::Error>> + Send;

    fn fetch_all(&self) -> impl Future<Output = Result<Vec<Sensor>, Self::Error>> + Send;

    fn fetch_history(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Vec<SensorEntry>, Self::Error>> + Send;
}
