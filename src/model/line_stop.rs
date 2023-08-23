use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use super::enums::Direction;

#[derive(Serialize, FromRow)]
pub struct LineStop {
    pub id: String,
    pub line_id: String,
    pub stop_id: String,
    pub direction: Direction,
    pub sequence: u8,
    pub variation: String,
    pub duration: u8,
}

impl LineStop {
    pub fn new(
        line_id: String,
        stop_id: String,
        direction: Direction,
        sequence: u8,
        variation: String,
        duration: u8,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            line_id,
            stop_id,
            direction,
            sequence,
            variation,
            duration,
        }
    }
}
