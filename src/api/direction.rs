use actix_web::{
    error::ResponseError,
    get,
    http::{header::ContentType, StatusCode},
    web::Data,
    web::Json,
    web::Path,
    HttpResponse,
};
use derive_more::Display;
use serde::Deserialize;

use crate::{
    model::{direction::Direction, direction_leg::DirectionLeg, leg_step::LegStep},
    repository::database::{Database, Table},
};

#[derive(Deserialize)]
pub struct DirectionIdentifier {
    id: String,
}

#[derive(Debug, Display)]
pub enum DirectionError {
    DirectionNotFound,
    BadDirectionRequest,
}

impl ResponseError for DirectionError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            DirectionError::DirectionNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/direction/{id}")]
pub async fn get_direction(
    identifier: Path<DirectionIdentifier>,
    database: Data<Database>,
) -> Result<Json<Direction>, DirectionError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(DirectionError::BadDirectionRequest);
    }

    let direction: Option<Direction> = database
        .get_one::<Direction>(
            sqlx::query_as::<_, Direction>(
                format!("SELECT * FROM {} WHERE id=$1", Direction::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
        )
        .await;

    match direction {
        Some(direction) => Ok(Json(direction)),
        None => Err(DirectionError::DirectionNotFound),
    }
}

#[get("/direction/{id}/legs")]
pub async fn get_direction_legs(
    identifier: Path<DirectionIdentifier>,
    database: Data<Database>,
) -> Result<Json<Vec<DirectionLeg>>, DirectionError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(DirectionError::BadDirectionRequest);
    }

    let direction_legs: Option<Vec<DirectionLeg>> = database
        .get_many::<DirectionLeg>(
            sqlx::query_as::<_, DirectionLeg>(
                format!("SELECT * FROM {} WHERE direction_id=$1", DirectionLeg::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
        )
        .await;

    match direction_legs {
        Some(direction_legs) => Ok(Json(direction_legs)),
        None => Err(DirectionError::DirectionNotFound),
    }
}

#[get("/direction/{id}/legs/steps")]
pub async fn get_direction_leg_steps(
    identifier: Path<DirectionIdentifier>,
    database: Data<Database>,
) -> Result<Json<Vec<LegStep>>, DirectionError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(DirectionError::BadDirectionRequest);
    }

    let direction_legs: Option<Vec<DirectionLeg>> = database
        .get_many::<DirectionLeg>(
            sqlx::query_as::<_, DirectionLeg>(
                format!("SELECT * FROM {} WHERE direction_id=$1", DirectionLeg::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
        )
        .await;

    if direction_legs.is_none() {
        return Err(DirectionError::DirectionNotFound);
    }

    let mut leg_steps: Vec<LegStep> = Vec::new();
    for leg in direction_legs.unwrap() {
        let lsteps: Vec<LegStep> = database
            .get_many::<LegStep>(
                sqlx::query_as::<_, LegStep>(
                    format!("SELECT * FROM {} WHERE leg_id=$1", LegStep::TABLE_NAME).as_str(),
                )
                .bind(leg.id),
            )
            .await.unwrap();
        leg_steps.extend(lsteps);
    }

    Ok(Json(leg_steps))
}