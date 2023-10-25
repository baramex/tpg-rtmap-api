use std::str::FromStr;

use async_trait::async_trait;
use chrono::NaiveTime;
use serde::Serialize;
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct TripStop {
    pub id: i32,
    pub stop_id: i32,
    pub trip_id: i32,
    pub sequence: i16,
    pub arrival_time: Option<NaiveTime>,
    pub departure_time: Option<NaiveTime>,
}

#[async_trait]
impl Table for TripStop {
    const TABLE_NAME: &'static str = "trip_stops";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.stop_id),
            Box::new(self.trip_id),
            Box::new(self.sequence),
            Box::new(if self.arrival_time.is_some() {
                self.arrival_time.unwrap()
            } else {
                NaiveTime::from_str("00:00:00").unwrap()
            }),
            Box::new(if self.departure_time.is_some() {
                self.departure_time.unwrap()
            } else {
                NaiveTime::from_str("00:00:00").unwrap()
            }),
        ]
    }

    fn keys() -> String {
        return "(id,stop_id,trip_id,sequence,arrival_time,departure_time)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            stop_id INTEGER NOT NULL,
            trip_id INTEGER NOT NULL,
            sequence SMALLINT NOT NULL,
            arrival_time SMALLINT,
            departure_time SMALLINT,
            CONSTRAINT fk_trip
                FOREIGN KEY(trip_id)
                    REFERENCES trips(id)
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
