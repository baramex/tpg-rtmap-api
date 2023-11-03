use async_trait::async_trait;
use serde::Serialize;
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct LegStep {
    pub id: i32,
    pub leg_id: i32,
    pub distance: f64,
    pub duration: i32,
    pub sequence: i16,
    pub start_lat: f64,
    pub start_lng: f64,
    pub end_lat: f64,
    pub end_lng: f64,
    pub polyline_lat: f64,
    pub polyline_lng: f64
}

#[async_trait]
impl Table for LegStep {
    const TABLE_NAME: &'static str = "leg_steps";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.leg_id),
            Box::new(self.distance),
            Box::new(self.duration),
            Box::new(self.sequence),
            Box::new(self.start_lat),
            Box::new(self.start_lng),
            Box::new(self.end_lat),
            Box::new(self.end_lng),
            Box::new(self.polyline_lat),
            Box::new(self.polyline_lng)
        ]
    }

    fn keys() -> String {
        return "(id,leg_id,distance,duration,sequence,start_lat,start_lng,end_lat,end_lng,polyline_lat,polyline_lng)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            leg_id INTEGER NOT NULL,
            distance DOUBLE PRECISION NOT NULL,
            duration INTEGER NOT NULL,
            sequence SMALLINT NOT NULL,
            start_lat DOUBLE PRECISION NOT NULL,
            start_lng DOUBLE PRECISION NOT NULL,
            end_lat DOUBLE PRECISION NOT NULL,
            end_lng DOUBLE PRECISION NOT NULL,
            polyline_lat DOUBLE PRECISION NOT NULL,
            polyline_lng DOUBLE PRECISION NOT NULL,
            CONSTRAINT fk_step
                FOREIGN KEY(step_id)
                    REFERENCES steps(id)
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
