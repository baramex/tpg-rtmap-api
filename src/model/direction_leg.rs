use async_trait::async_trait;
use serde::Serialize;
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct DirectionLeg {
    pub id: i32,
    pub direction_id: i32,
    pub distance: i32,
    pub duration: i32,
    pub sequence: i16,
    pub origin_id: i32,
    pub destination_id: i32,
}

#[async_trait]
impl Table for DirectionLeg {
    const TABLE_NAME: &'static str = "direction_legs";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.direction_id),
            Box::new(self.distance),
            Box::new(self.duration),
            Box::new(self.sequence),
            Box::new(self.origin_id),
            Box::new(self.destination_id),
        ]
    }

    fn keys() -> String {
        return "(id,direction_id,distance,duration,sequence,origin_id,destination_id)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            direction_id INTEGER NOT NULL,
            distance DOUBLE PRECISION NOT NULL,
            duration INTEGER NOT NULL,
            sequence SMALLINT NOT NULL,
            origin_id INTEGER NOT NULL,
            destination_id INTEGER NOT NULL,
            CONSTRAINT fk_origin
                FOREIGN KEY(origin_id)
                    REFERENCES stops(id),
            CONSTRAINT fk_destination
                FOREIGN KEY(destination_id)
                    REFERENCES stops(id),
            CONSTRAINT fk_direction
                FOREIGN KEY(direction_id)
                    REFERENCES directions(id)
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
