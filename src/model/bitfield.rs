use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, Error, postgres::PgQueryResult};

use crate::repository::database::{Database, Table, RowData};

#[derive(Serialize, FromRow, Debug)]
pub struct Bitfield {
    pub id: u32,
    pub days: String,
}

#[async_trait]
impl Table for Bitfield {
    const TABLE_NAME: &'static str = "bitfields";
    
    fn format(&self) -> String {
        format!(
            "({},'{}')",
            self.id,
            self.days
        )
    }

    fn values(&self) -> Vec<RowData> {
        vec![
            self.id,
            self.days,
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