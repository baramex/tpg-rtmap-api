use async_trait::async_trait;
use serde::Serialize;
use sqlx::{
    postgres::{PgQueryResult, PgRow},
    Error, FromRow, Row,
};

use crate::repository::database::{Database, Table};

use super::types::Hour;

#[derive(Serialize, Debug)]
pub struct TripStop {
    pub id: i32,
    pub stop_id: i32,
    pub trip_id: i32,
    pub sequence: i16,
    pub arrival_time: Option<Hour>,
    pub departure_time: Option<Hour>,
}

impl<'r> FromRow<'r, PgRow> for TripStop {
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        let arrival_time: Option<i16> = row.try_get("arrival_time")?;
        let departure_time: Option<i16> = row.try_get("departure_time")?;

        Ok(TripStop {
            id: row.try_get("id")?,
            stop_id: row.try_get("stop_id")?,
            trip_id: row.try_get("trip_id")?,
            sequence: row.try_get("sequence")?,
            arrival_time: if arrival_time.is_none() {
                None
            } else {
                Some(Hour::try_from(arrival_time.unwrap()).unwrap())
            },
            departure_time: if departure_time.is_none() {
                None
            } else {
                Some(Hour::try_from(departure_time.unwrap()).unwrap())
            },
        })
    }
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
                self.arrival_time.as_ref().unwrap().value()
            } else {
                0
            }),
            Box::new(if self.departure_time.is_some() {
                self.departure_time.as_ref().unwrap().value()
            } else {
                0
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
