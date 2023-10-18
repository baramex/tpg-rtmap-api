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
    model::stop::Stop,
    repository::database::{Database, Table},
};

#[derive(Deserialize, Serialize)]
pub struct StopIdentifier {
    id: String,
}

#[derive(Debug, Display)]
pub enum StopError {
    StopNotFound,
    BadStopRequest,
}

impl ResponseError for StopError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            StopError::StopNotFound => StatusCode::NOT_FOUND,
            StopError::BadStopRequest => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/stop/{id}")]
pub async fn get_stop(
    identifier: Path<StopIdentifier>,
    database: Data<Database>,
) -> Result<Json<Stop>, StopError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(StopError::BadStopRequest);
    }

    let stop: Option<Stop> = database
        .get_one::<Stop>(
            sqlx::query_as::<_, Stop>(format!("SELECT * FROM {} WHERE id=$1", Stop::TABLE_NAME).as_str()).bind(id.unwrap())
        )
        .await;

    match stop {
        Some(stop) => Ok(Json(stop)),
        None => Err(StopError::StopNotFound),
    }
}
