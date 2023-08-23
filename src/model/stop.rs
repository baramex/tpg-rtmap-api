use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, FromRow, Debug, Clone)]
pub struct Stop {
    pub id: String,
    pub reference: String,
    pub lat: f64,
    pub lon: f64,
    pub name: String,
}

impl Stop {
    pub fn new(reference: String, lat: f64, lon: f64, name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            reference,
            lat,
            lon,
            name,
        }
    }
}
