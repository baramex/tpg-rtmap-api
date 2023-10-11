use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow, Debug, Clone)]
pub struct Bitfield {
    pub id: u32,
    pub days: String,
}