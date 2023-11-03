use async_trait::async_trait;
use serde::Serialize;
use sqlx::{postgres::PgQueryResult, Error, FromRow};

use crate::repository::database::{Database, Table};

#[derive(Serialize, FromRow, Debug)]
pub struct StepPath {
    pub id: i32,
    pub step_id: i32,
    pub latitude: f64,
    pub longitude: f64,
}

#[async_trait]
impl Table for StepPath {
    const TABLE_NAME: &'static str = "step_paths";

    fn values(&self) -> Vec<Box<dyn std::any::Any>> {
        vec![
            Box::new(self.id),
            Box::new(self.step_id),
            Box::new(self.latitude),
            Box::new(self.longitude),
        ]
    }

    fn keys() -> String {
        return "(id,step_id,latitude,longitude)".to_string();
    }

    async fn create_table(database: &Database) -> Result<PgQueryResult, Error> {
        database
            .query(
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY,
            step_id INTEGER NOT NULL,
            latitude DOUBLE PRECISION NOT NULL,
            longitude DOUBLE PRECISION NOT NULL,
            CONSTRAINT fk_step
                FOREIGN KEY(step_id)
                    REFERENCES leg_steps(id)
        )",
                    Self::TABLE_NAME
                )
                .as_str(),
            )
            .await
    }
}
