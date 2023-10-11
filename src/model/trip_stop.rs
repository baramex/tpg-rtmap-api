use serde::Serialize;
use sqlx::FromRow;

use super::{line::TransportMode, enums::Direction};

#[derive(Serialize, FromRow, Debug)]
pub struct TripStop {
    pub id: u32,
    pub trip_id: u32,
    pub sequence: u8,
    pub arrival_time: String,
    pub departure_time: String,
}