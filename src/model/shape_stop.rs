use async_trait::async_trait;
use serde::Serialize;
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct ShapeStop {
    pub id: i32,
    pub shape_id: i32,
    pub stop_id: i32,
    pub sequence: i16,
}

#[async_trait]
impl Table for ShapeStop {
    const TABLE_NAME: &'static str = "shape_stops";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.shape_id),
            Box::new(self.stop_id),
            Box::new(self.sequence),
        ]
    }

    fn keys() -> String {
        return "(id,shape_id,stop_id,sequence)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            shape_id INTEGER NOT NULL,
            stop_id INTEGER NOT NULL,
            sequence SMALLINT NOT NULL,
            CONSTRAINT fk_stop
                FOREIGN KEY(stop_id)
                    REFERENCES stops(id),
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
