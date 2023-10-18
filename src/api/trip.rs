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
use serde::{Deserialize, Serialize};

use crate::{
    model::{trip::Trip, trip_stop::TripStop},
    repository::database::{Database, Table},
};

#[derive(Deserialize, Serialize)]
pub struct TripIdentifier {
    id: String,
}

#[derive(Debug, Display)]
pub enum TripError {
    TripNotFound,
    BadTripRequest,
}

impl ResponseError for TripError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            TripError::TripNotFound => StatusCode::NOT_FOUND,
            TripError::BadTripRequest => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/trip/{id}")]
pub async fn get_trip(
    identifier: Path<TripIdentifier>,
    database: Data<Database>,
) -> Result<Json<Trip>, TripError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(TripError::BadTripRequest);
    }

    let trip: Option<Trip> = database
        .get_one::<Trip>(
            sqlx::query_as::<_, Trip>(format!("SELECT * FROM {} WHERE id=$1", Trip::TABLE_NAME).as_str()).bind(id.unwrap())
        )
        .await;

    match trip {
        Some(trip) => Ok(Json(trip)),
        None => Err(TripError::TripNotFound),
    }
}

#[get("/trip/{id}/stops")]
pub async fn get_trip_stops(
    identifier: Path<TripIdentifier>,
    database: Data<Database>,
) -> Result<Json<Vec<TripStop>>, TripError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(TripError::BadTripRequest);
    }

    let trip_stops: Option<Vec<TripStop>> = database
        .get::<TripStop>(
            sqlx::query_as::<_, TripStop>(format!("SELECT * FROM {} WHERE trip_id=$1", TripStop::TABLE_NAME).as_str()).bind(id.unwrap())
        )
        .await;

    match trip_stops {
        Some(trip_stops) => Ok(Json(trip_stops)),
        None => Err(TripError::TripNotFound),
    }
}