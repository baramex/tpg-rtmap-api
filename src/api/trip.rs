use std::ops::AddAssign;

use actix_web::{
    error::ResponseError,
    get,
    http::{header::ContentType, StatusCode},
    web::Data,
    web::Path,
    web::{Json, Query},
    HttpResponse,
};
use chrono::{Datelike, Duration, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::Europe::Zurich;
use derive_more::Display;
use serde::Deserialize;
use std::ops::SubAssign;

use crate::{
    model::{information::Information, trip::Trip, trip_stop::TripStop},
    repository::database::{Database, Table},
};

#[derive(Deserialize)]
pub struct TripIdentifier {
    id: String,
}

#[derive(Deserialize)]
pub struct TripSelector {
    timestamp: i64,
    bounds: Option<i16>,
    from: Option<i64>,
}

#[derive(Debug, Display)]
pub enum TripError {
    TripNotFound,
    BadTripRequest,
    InvalidTimePeriod,
    InvalidBounds,
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
            _ => StatusCode::BAD_REQUEST,
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

    let bounds: i16 = info.bounds.unwrap_or(0);
    if bounds > 24 || bounds < 0 {
        return Err(TripError::InvalidBounds);
    }

    let from: Option<i64> = info.from;

    let date = Zurich.from_utc_datetime(&naive_date.unwrap());
    let information: Information = database
        .get_one::<Information>(sqlx::query_as::<_, Information>(
            format!("SELECT * FROM {}", Information::TABLE_NAME).as_str(),
        ))
        .await
        .unwrap();

    let start_datetime = Zurich
        .with_ymd_and_hms(
            information.start_date.year(),
            information.start_date.month(),
            information.start_date.day(),
            0,
            0,
            0,
        )
        .unwrap();
    let end_datetime = Zurich
        .with_ymd_and_hms(
            information.end_date.year(),
            information.end_date.month(),
            information.end_date.day(),
            23,
            59,
            59,
        )
        .unwrap();

    if date.lt(&start_datetime) || date.gt(&end_datetime) {
        return Err(TripError::InvalidTimePeriod);
    }

    let mut date_from = start_datetime;
    if from.is_some() {
        let naive_from: Option<NaiveDateTime> = NaiveDateTime::from_timestamp_opt(from.unwrap(), 0);
        if naive_from.is_none() {
            return Err(TripError::BadTripRequest);
        }

        date_from = Zurich.from_utc_datetime(&naive_from.unwrap());
        if date_from.lt(&start_datetime) || date_from.gt(&end_datetime) {
            return Err(TripError::InvalidTimePeriod);
        }
    }

    let mut upper_time_bound: NaiveTime = date.time();
    upper_time_bound.add_assign(Duration::minutes(bounds as i64));
    let mut lower_time_bound: NaiveTime = date.time();
    lower_time_bound.sub_assign(Duration::minutes(bounds as i64));

    let day_number: i16 = date.signed_duration_since(start_datetime).num_days() as i16;
    let bitfield_number: i16 = day_number + 2;

    // TODO: manage trips that are after 00h

    let trips: Option<Vec<Trip>> = database.get_many::<Trip>(sqlx::query_as::<_, Trip>(format!("SELECT trips.id, trips.journey_number, trips.option_count, trips.transport_mode, trips.origin_id, trips.destination_id, trips.bitfield_id, trips.line_id, trips.direction, trips.departure_time, trips.arrival_time FROM {} JOIN bitfields ON bitfield_id = bitfields.id WHERE departure_time <= $1 AND departure_time >= $4 AND arrival_time >= $2 AND SUBSTRING(days,$3,1) = '1'", Trip::TABLE_NAME).as_str()).bind(upper_time_bound).bind(lower_time_bound).bind(bitfield_number+1).bind(date_from.time())).await;

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
