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
    model::{shape::Shape, shape_point::ShapePoint, shape_stop::ShapeStop},
    repository::database::{Database, Table},
};

#[derive(Deserialize)]
pub struct ShapeIdentifier {
    id: String,
}

#[derive(Debug, Display)]
pub enum ShapeError {
    ShapeNotFound,
    BadShapeRequest,
}

impl ResponseError for ShapeError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ShapeError::ShapeNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/shape/{id}")]
pub async fn get_shape(
    identifier: Path<ShapeIdentifier>,
    database: Data<Database>,
) -> Result<Json<Shape>, ShapeError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(ShapeError::BadShapeRequest);
    }

    let shape: Option<Shape> = database
        .get_one::<Shape>(
            sqlx::query_as::<_, Shape>(
                format!("SELECT * FROM {} WHERE id=$1", Shape::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
        )
        .await;

    match shape {
        Some(shape) => Ok(Json(shape)),
        None => Err(ShapeError::ShapeNotFound),
    }
}

#[get("/shape/{id}/points")]
pub async fn get_shape_points(
    identifier: Path<ShapeIdentifier>,
    database: Data<Database>,
) -> Result<Json<Vec<ShapePoint>>, ShapeError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(ShapeError::BadShapeRequest);
    }

    let shape_points: Option<Vec<ShapePoint>> = database
        .get_many::<ShapePoint>(
            sqlx::query_as::<_, ShapePoint>(
                format!("SELECT * FROM {} WHERE shape_id=$1", ShapePoint::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
        )
        .await;

    match shape_points {
        Some(shape_points) => Ok(Json(shape_points)),
        None => Err(ShapeError::ShapeNotFound),
    }
}

#[get("/shape/{id}/stops")]
pub async fn get_shape_stops(
    identifier: Path<ShapeIdentifier>,
    database: Data<Database>,
) -> Result<Json<Vec<ShapeStop>>, ShapeError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(ShapeError::BadShapeRequest);
    }

    let shape_stops: Option<Vec<ShapeStop>> = database
        .get_many::<ShapeStop>(
            sqlx::query_as::<_, ShapeStop>(
                format!("SELECT * FROM {} WHERE shape_id=$1", ShapeStop::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
        )
        .await;

    match shape_stops {
        Some(shape_stops) => Ok(Json(shape_stops)),
        None => Err(ShapeError::ShapeNotFound),
    }
}