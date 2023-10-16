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
    pub arrival_time: Hour,
    pub departure_time: Hour,
}

#[async_trait]
impl Table for  TripStop {
    const TABLE_NAME: &'static str = "trip_stops";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.trip_id),
            Box::new(self.sequence),
            Box::new(self.arrival_time.value()),
            Box::new(self.departure_time.value()),
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
            arrival_time VARCHAR(5),
            departure_time VARCHAR(5),
            CONSTRAINT fk_trip
                FOREIGN KEY(trip_id)
                    REFERENCES trips(id)
                    ON DELETE CASCADE
        )", Self::TABLE_NAME).as_str()).await
    }
}