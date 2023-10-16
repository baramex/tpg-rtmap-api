use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, postgres::PgQueryResult, Error};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct Stop {
    pub id: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub name: String,
}

#[async_trait]
impl Table for  Stop {
    const TABLE_NAME: &'static str = "stops";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.latitude),
            Box::new(self.longitude),
            Box::new(self.name.to_string()),
        ]
    }

    fn keys() -> String {
        return "(id,latitude,longitude,name)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database.query(format!("CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            name VARCHAR(60) NOT NULL
        )", Self::TABLE_NAME).as_str()).await
    }
}