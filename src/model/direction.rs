use async_trait::async_trait;
use serde::Serialize;
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct Direction {
    pub id: i32,
    pub identifier: String,
    pub origin_id: i32,
    pub destination_id: i32,
}

#[async_trait]
impl Table for Direction {
    const TABLE_NAME: &'static str = "directions";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.identifier.to_string()),
            Box::new(self.origin_id),
            Box::new(self.destination_id),
        ]
    }

    fn keys() -> String {
        return "(id,identifier,origin_id,destination_id)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            identifier VARCHAR(1024) NOT NULL,
            origin_id INTEGER NOT NULL,
            destination_id INTEGER NOT NULL,
            CONSTRAINT fk_origin
                FOREIGN KEY(origin_id)
                    REFERENCES stops(id),
            CONSTRAINT fk_destination
                FOREIGN KEY(destination_id)
                    REFERENCES stops(id)
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}