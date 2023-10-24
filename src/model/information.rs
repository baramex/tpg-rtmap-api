use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::{
    postgres::PgQueryResult,
    Error, FromRow
};

use crate::repository::database::{Database, Table};

#[derive(Serialize, Debug, FromRow)]
pub struct Information {
    pub id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[async_trait]
impl Table for Information {
    const TABLE_NAME: &'static str = "information";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.start_date),
            Box::new(self.end_date),
        ]
    }

    fn keys() -> String {
        return "(id,start_date,end_date)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id SERIAL PRIMARY KEY,
            start_date DATE NOT NULL,
            end_date DATE NOT NULL
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
