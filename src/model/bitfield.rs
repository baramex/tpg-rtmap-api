use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, Error, postgres::PgQueryResult};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct Bitfield {
    pub id: i32,
    pub days: String,
}

#[async_trait]
impl Table for Bitfield {
    const TABLE_NAME: &'static str = "bitfields";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.days.to_string()),
        ]
    }

    fn keys() -> String {
        return "(id,days)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database.query(format!("CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            days VARCHAR(100) NOT NULL
        )", Self::TABLE_NAME).as_str()).await
    }
}