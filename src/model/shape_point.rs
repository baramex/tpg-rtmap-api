use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

#[derive(Debug, Deserialize, Serialize)]
pub struct RoadResponse {
    pub snappedPoints: Vec<SnappedPoint>,
    pub warningMessage: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SnappedPoint {
    pub location: Location,
    pub originalIndex: Option<i32>,
    pub placeId: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, FromRow, Debug)]
pub struct ShapePoint {
    pub id: i32,
    pub shape_id: i32,
    pub sequence: i16,
    pub latitude: f64,
    pub longitude: f64,
    pub shape_stop_id: Option<i32>,
}

#[async_trait]
impl Table for ShapePoint {
    const TABLE_NAME: &'static str = "shape_points";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.shape_id),
            Box::new(self.sequence),
            Box::new(self.latitude),
            Box::new(self.longitude),
            Box::new(if self.shape_stop_id.is_some() {
                self.shape_stop_id.unwrap()
            } else {
                0
            }),
        ]
    }

    fn keys() -> String {
        return "(id,shape_id,sequence,latitude,longitude,shape_stop_id)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            shape_id INTEGER NOT NULL,
            sequence SMALLINT NOT NULL,
            latitude DOUBLE PRECISION NOT NULL,
            longitude DOUBLE PRECISION NOT NULL,
            shape_stop_id INTEGER,
            CONSTRAINT fk_shape_stop
                FOREIGN KEY(shape_stop_id)
                    REFERENCES shape_stops(id),
            CONSTRAINT fk_shape
                FOREIGN KEY(shape_id)
                    REFERENCES shapes(id)
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
