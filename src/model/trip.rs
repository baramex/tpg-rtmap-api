use serde::Serialize;
use sqlx::FromRow;

use super::{line::TransportMode, enums::Direction};

#[derive(Serialize, FromRow, Debug)]
pub struct Trip {
    id: u32,
    journey_number: u32,
    option_count: u16,
    transport_mode: TransportMode,
    origin: u32,
    destination: u32,
    bitfield_id: u32,
    line_id: u32,
    direction: Direction,
    departure_time: String,
    arrival_time: String,
}