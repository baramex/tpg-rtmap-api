use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, postgres::PgQueryResult, Error};

use crate::repository::database::{Database, Table};

use super::types::Hour;

#[derive(Serialize, FromRow, Debug)]
pub struct TripStop {
    pub id: i32,
    pub trip_id: i32,
    pub sequence: i8,
    pub arrival_time: Option<Hour>,
    pub departure_time: Option<Hour>,
}

#[async_trait]
impl Table for  TripStop {
    const TABLE_NAME: &'static str = "trip_stops";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.trip_id),
            Box::new(self.sequence),
            Box::new(if self.arrival_time.is_some() { self.arrival_time.as_ref().unwrap().value() } else { 0 }),
            Box::new(if self.departure_time.is_some() { self.departure_time.as_ref().unwrap().value() } else { 0 }),
        ]
    }

    fn keys() -> String {
        return "(id,trip_id,sequence,arrival_time,departure_time)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database.query(format!("CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            trip_id INTEGER NOT NULL,
            sequence SMALLINT NOT NULL,
            arrival_time SMALLINT,
            departure_time SMALLINT,
            CONSTRAINT fk_trip
                FOREIGN KEY(trip_id)
                    REFERENCES trips(id)
                    ON DELETE CASCADE
        )", Self::TABLE_NAME).as_str()).await
    }
}