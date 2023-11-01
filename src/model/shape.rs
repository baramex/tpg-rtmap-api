use async_trait::async_trait;
use serde::Serialize;
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct Shape {
    pub id: i32,
    pub identifier: String
}

#[async_trait]
impl Table for Shape {
    const TABLE_NAME: &'static str = "shapes";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.identifier.to_string()),
        ]
    }

    fn keys() -> String {
        return "(id,identifier)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            identifier VARCHAR(512) NOT NULL
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
