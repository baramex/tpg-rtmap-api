use async_trait::async_trait;
use serde::Serialize;
use sqlx::{FromRow, Error, postgres::PgQueryResult};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct Bitfield {
    pub id: i32,
    pub days: String,
}

impl Bitfield {
    pub fn convert_hex_to_bits(hex: &str) -> String {
        let mut bits: String = String::new();
        for c in hex.chars() {
            let mut bit: String = format!("{:b}", u32::from_str_radix(&c.to_string(), 16).unwrap());
            while bit.len() < 4 {
                bit = format!("0{}", bit);
            }
            bits.push_str(bit.as_str());
        }
        return bits;
    }
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
            days VARCHAR(400) NOT NULL
        )", Self::TABLE_NAME).as_str()).await
    }
}