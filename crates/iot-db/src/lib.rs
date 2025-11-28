mod errors;

use iot_entities::{Sensor, SensorEntry, SensorService};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::errors::DbError;

#[derive(Clone)]
pub struct SensorModel {
    db: Pool<MySql>,
}

impl From<Pool<MySql>> for SensorModel {
    fn from(db: Pool<MySql>) -> Self {
        Self { db }
    }
}

impl SensorService for SensorModel {
    type Error = DbError;

    async fn save_entry(&self, sensor_id: Uuid, ppm: f64) -> Result<SensorEntry, Self::Error> {
        let entry_id = Uuid::now_v7();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let query = sqlx::query(
            "INSERT INTO sensor_entries (id, sensor_id, value, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&entry_id)
        .bind(&sensor_id)
        .bind(ppm)
        .bind(now)
        .execute(&self.db)
        .await?;

        if query.rows_affected() < 1 {
            return Err(DbError::NotInserted);
        }

        Ok(SensorEntry {
            id: entry_id,
            sensor_id,
            value: ppm,
            created_at: now,
        })
    }

    async fn register_sensor(&self, id: Uuid, label: String) -> Result<Sensor, Self::Error> {
        let sensor = sqlx::query_as::<_, Sensor>("SELECT * FROM sensor WHERE id = ?")
            .bind(&id)
            .fetch_optional(&self.db)
            .await?;

        match sensor {
            Some(sensor) => return Ok(sensor),
            None => {
                let now = OffsetDateTime::now_utc().unix_timestamp();

                let _ = sqlx::query("INSERT INTO sensor (id, label, created_at) VALUES (?, ?, ?)")
                    .bind(&id)
                    .bind(&label)
                    .bind(now)
                    .execute(&self.db)
                    .await?;

                Ok(Sensor {
                    id,
                    label,
                    created_at: now,
                })
            }
        }
    }

    async fn fetch_one(&self, id: Uuid) -> Result<Sensor, Self::Error> {
        let sensors = sqlx::query_as::<_, Sensor>("SELECT * FROM sensor WHERE id = ?")
            .bind(id)
            .fetch_one(&self.db)
            .await?;

        Ok(sensors)
    }

    async fn fetch_all(&self) -> Result<Vec<Sensor>, Self::Error> {
        let sensors = sqlx::query_as::<_, Sensor>("SELECT * FROM sensor")
            .fetch_all(&self.db)
            .await?;

        Ok(sensors)
    }

    async fn fetch_history(&self, id: Uuid) -> Result<Vec<SensorEntry>, Self::Error> {
        let entries =
            sqlx::query_as::<_, SensorEntry>("SELECT * FROM sensor_entries WHERE sensor_id = ?")
                .bind(id)
                .fetch_all(&self.db)
                .await?;

        Ok(entries)
    }
}

pub async fn connect_database(database_url: &str) -> Result<Pool<MySql>, DbError> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(pool)
}
