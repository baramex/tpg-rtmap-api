use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, postgres::PgQueryResult, Error};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct TripStop {
    pub id: u32,
    pub trip_id: u32,
    pub sequence: u8,
    pub arrival_time: String,
    pub departure_time: String,
}

#[async_trait]
impl Table for  TripStop {
    const TABLE_NAME: &'static str = "trip_stops";

    fn format(&self) -> String {
        format!(
            "({},{},{},'{}','{}')",
            self.id,
            self.trip_id,
            self.sequence,
            self.arrival_time,
            self.departure_time
        )
    }

    fn keys() -> String {
        return "(id,trip_id,sequence,arrival_time,departure_time)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database.query(format!("CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            trip_id INTEGER NOT NULL,
            sequence SMALLINT NOT NULL,
            arrival_time VARCHAR(5) NOT NULL,
            departure_time VARCHAR(5) NOT NULL,
            CONSTRAINT fk_trip
                FOREIGN KEY(trip_id)
                    REFERENCES trips(id)
                    ON DELETE CASCADE
        )", Self::TABLE_NAME).as_str()).await
    }
}