use async_trait::async_trait;
use chrono::NaiveTime;
use serde::Serialize;
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

use super::{line::TransportMode, types::Direction};

#[derive(Serialize, FromRow, Debug)]
pub struct Trip {
    pub id: i32,
    pub journey_number: i32,
    pub option_count: i16,
    #[sqlx(try_from = "String")]
    pub transport_mode: TransportMode,
    pub origin_id: i32,
    pub destination_id: i32,
    pub bitfield_id: i32,
    pub line_id: i32,
    #[sqlx(try_from = "String")]
    pub direction: Direction,
    pub departure_time: NaiveTime,
    pub arrival_time: NaiveTime,
}

#[async_trait]
impl Table for Trip {
    const TABLE_NAME: &'static str = "trips";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.journey_number),
            Box::new(self.option_count),
            Box::new(format!("{:?}", self.transport_mode)),
            Box::new(self.origin_id),
            Box::new(self.destination_id),
            Box::new(self.bitfield_id),
            Box::new(self.line_id),
            Box::new(format!("{:?}", self.direction)),
            Box::new(self.departure_time),
            Box::new(self.arrival_time),
        ]
    }

    fn keys() -> String {
        return "(id,journey_number,option_count,transport_mode,origin_id,destination_id,bitfield_id,line_id,direction,departure_time,arrival_time)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            journey_number INTEGER NOT NULL,
            option_count SMALLINT NOT NULL,
            transport_mode VARCHAR(12) NOT NULL,
            origin_id INTEGER NOT NULL,
            destination_id INTEGER NOT NULL,
            bitfield_id INTEGER NOT NULL,
            line_id INTEGER NOT NULL,
            direction VARCHAR(7) NOT NULL,
            departure_time TIME NOT NULL,
            arrival_time TIME NOT NULL,
            CONSTRAINT fk_origin
                FOREIGN KEY(origin_id)
                    REFERENCES stops(id),
            CONSTRAINT fk_destination
                FOREIGN KEY(destination_id)
                    REFERENCES stops(id),
            CONSTRAINT fk_bitfield
                FOREIGN KEY(bitfield_id)
                    REFERENCES bitfields(id),
            CONSTRAINT fk_line
                FOREIGN KEY(line_id)
                    REFERENCES lines(id)
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
