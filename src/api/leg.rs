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
    model::{leg_step::LegStep, direction_leg::DirectionLeg},
    repository::database::{Database, Table},
};

#[derive(Deserialize)]
pub struct LegIdentifier {
    id: String,
}

#[derive(Debug, Display)]
pub enum LegError {
    LegNotFound,
    BadLegRequest,
}

impl ResponseError for LegError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            LegError::LegNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/leg/{id}")]
pub async fn get_leg(
    identifier: Path<LegIdentifier>,
    database: Data<Database>,
) -> Result<Json<DirectionLeg>, LegError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(LegError::BadLegRequest);
    }

    let leg: Option<DirectionLeg> = database
        .get_one::<DirectionLeg>(
            sqlx::query_as::<_, DirectionLeg>(
                format!("SELECT * FROM {} WHERE id=$1", DirectionLeg::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
        )
        .await;

    match leg {
        Some(leg) => Ok(Json(leg)),
        None => Err(LegError::LegNotFound),
    }
}

#[get("/leg/{id}/steps")]
pub async fn get_leg_steps(
    identifier: Path<LegIdentifier>,
    database: Data<Database>,
) -> Result<Json<Vec<LegStep>>, LegError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(LegError::BadLegRequest);
    }

    let leg_steps: Option<Vec<LegStep>> = database
        .get_many::<LegStep>(
            sqlx::query_as::<_, LegStep>(
                format!("SELECT * FROM {} WHERE leg_id=$1", LegStep::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
        )
        .await;

    match leg_steps {
        Some(leg_steps) => Ok(Json(leg_steps)),
        None => Err(LegError::LegNotFound),
    }
}