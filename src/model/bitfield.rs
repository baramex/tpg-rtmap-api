use serde::Serialize;
use sqlx::{FromRow, Error, postgres::PgQueryResult};

use crate::repository::database::Database;

#[derive(Serialize, FromRow, Debug)]
pub struct Bitfield {
    pub id: u32,
    pub days: String,
}

impl Bitfield {
    pub async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database.query("CREATE TABLE IF NOT EXISTS bitfield (
            id INTEGER PRIMARY KEY,
            days VARCHAR NOT NULL
        )").await
    }
}