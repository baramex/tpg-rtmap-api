use std::{env, time::Instant};

use actix_web::{
    error::ResponseError,
    get,
    http::{header::ContentType, StatusCode},
    web::Data,
    web::Path,
    web::{Json, Query},
    HttpResponse,
};
use chrono::{DateTime, NaiveDateTime, Timelike, Utc};
use derive_more::Display;
use serde::Deserialize;
use sqlx::{Postgres, QueryBuilder};

use crate::{
    model::{
        bitfield::Bitfield, information::Information, trip::Trip, trip_stop::TripStop, types::Hour,
    },
    repository::database::{Database, Table},
};

#[derive(Deserialize)]
pub struct TripIdentifier {
    id: String,
}

#[derive(Deserialize)]
pub struct TripSelector {
    timestamp: i64,
}

#[derive(Debug, Display)]
pub enum TripError {
    TripNotFound,
    BadTripRequest,
    InvalidTimePeriod,
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
            TripError::InvalidTimePeriod => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/trips")]
pub async fn get_trips(
    database: Data<Database>,
    info: Query<TripSelector>,
) -> Result<Json<Vec<Trip>>, TripError> {
    let timestamp: i64 = info.timestamp;

    let naive_date: Option<NaiveDateTime> = NaiveDateTime::from_timestamp_opt(timestamp, 0);
    if naive_date.is_none() {
        return Err(TripError::BadTripRequest);
    }

    let date: DateTime<Utc> = DateTime::from_utc(naive_date.unwrap(), Utc);
    let information: Information = database
        .get_one::<Information>(sqlx::query_as::<_, Information>(
            format!("SELECT * FROM {}", Information::TABLE_NAME).as_str(),
        ))
        .await
        .unwrap();

    let start_datetime: DateTime<Utc> =
        DateTime::from_utc(information.start_date.and_hms_opt(0, 0, 0).unwrap(), Utc);
    let end_datetime: DateTime<Utc> =
        DateTime::from_utc(information.end_date.and_hms_opt(23, 59, 59).unwrap(), Utc);

    if date.lt(&start_datetime) || date.gt(&end_datetime) {
        return Err(TripError::InvalidTimePeriod);
    }

    let hour = Hour {
        hour: i16::try_from(date.hour()).unwrap(),
        minute: i16::try_from(date.minute()).unwrap(),
    };

    let day_number: i16 = date.signed_duration_since(start_datetime).num_days() as i16;
    let bitfield_number: i16 = day_number + 2;

    let trips: Option<Vec<Trip>> = database
        .get_many::<Trip>(sqlx::query_as::<_, Trip>(format!("SELECT * FROM {} JOIN bitfields ON bitfield_id = bitfields.id WHERE departure_time <= $1 AND arrival_time >= $1 and SUBSTRING(days,$2,$2) = $3", Trip::TABLE_NAME).as_str()).bind(hour.value()).bind(bitfield_number+1).bind("1"))
        .await;

    match trips {
        Some(trips) => Ok(Json(trips)),
        None => Err(TripError::TripNotFound),
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
            sqlx::query_as::<_, Trip>(
                format!("SELECT * FROM {} WHERE id=$1", Trip::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
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
        .get_many::<TripStop>(
            sqlx::query_as::<_, TripStop>(
                format!("SELECT * FROM {} WHERE trip_id=$1", TripStop::TABLE_NAME).as_str(),
            )
            .bind(id.unwrap()),
        )
        .await;

    match trip_stops {
        Some(trip_stops) => Ok(Json(trip_stops)),
        None => Err(TripError::TripNotFound),
    }
}
