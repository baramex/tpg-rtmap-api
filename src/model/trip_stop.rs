use serde::Serialize;
use sqlx::FromRow;

use super::{line::TransportMode, enums::Direction};

#[derive(Serialize, FromRow, Debug)]
pub struct TripStop {
    id: u32,
    trip_id: u32,
    arrival_time: String,
    departure_time: String,
}