use async_trait::async_trait;
use serde::Serialize;
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

use super::{enums::Direction, line::TransportMode};

#[derive(Serialize, FromRow, Debug)]
pub struct Trip {
    pub id: u32,
    pub journey_number: u32,
    pub option_count: u16,
    pub transport_mode: TransportMode,
    pub origin_id: u32,
    pub destination_id: u32,
    pub bitfield_id: u32,
    pub line_id: u32,
    pub direction: Direction,
    pub departure_time: String,
    pub arrival_time: String,
}

#[async_trait]
impl Table for Trip {
    const TABLE_NAME: &'static str = "trips";

    fn format(&self) -> String {
        format!(
            "{},{},{},'{:?}',{},{},{},{},'{:?}','{}','{}'",
            self.id,
            self.journey_number,
            self.option_count,
            self.transport_mode,
            self.origin_id,
            self.destination_id,
            self.bitfield_id,
            self.line_id,
            self.direction,
            self.departure_time,
            self.arrival_time
        )
    }

    fn values(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.journey_number.to_string(),
            self.option_count.to_string(),
            format!("{:?}", self.transport_mode),
            self.origin_id.to_string(),
            self.destination_id.to_string(),
            self.bitfield_id.to_string(),
            self.line_id.to_string(),
            format!("{:?}", self.direction),
            self.departure_time.to_string(),
            self.arrival_time.to_string(),
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
            departure_time VARCHAR(5) NOT NULL,
            arrival_time VARCHAR(5) NOT NULL,
            CONSTRAINT fk_origin
                FOREIGN KEY(origin_id)
                    REFERENCES stops(id)
                    ON DELETE CASCADE,
            CONSTRAINT fk_destination
                FOREIGN KEY(destination_id)
                    REFERENCES stops(id)
                    ON DELETE CASCADE,
            CONSTRAINT fk_bitfield
                FOREIGN KEY(bitfield_id)
                    REFERENCES bitfields(id)
                    ON DELETE CASCADE,
            CONSTRAINT fk_line
                FOREIGN KEY(line_id)
                    REFERENCES lines(id)
                    ON DELETE CASCADE
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
