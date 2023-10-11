use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow, Debug)]
pub struct Stop {
    pub id: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub name: String,
}