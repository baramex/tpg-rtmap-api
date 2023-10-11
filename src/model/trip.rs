use serde::Serialize;
use sqlx::FromRow;

use super::{line::TransportMode, enums::Direction};

#[derive(Serialize, FromRow, Debug)]
pub struct Trip {
    pub id: u32,
    pub journey_number: u32,
    pub option_count: u16,
    pub transport_mode: TransportMode,
    pub origin: u32,
    pub destination: u32,
    pub bitfield_id: u32,
    pub line_id: u32,
    pub direction: Direction,
    pub departure_time: String,
    pub arrival_time: String,
}