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
    model::line::Line,
    repository::database::{Database, Table},
};

#[derive(Deserialize, Serialize)]
pub struct LineIdentifier {
    id: String,
}

#[derive(Debug, Display)]
pub enum LineError {
    LineNotFound,
    BadLineRequest,
}

impl ResponseError for LineError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            LineError::LineNotFound => StatusCode::NOT_FOUND,
            LineError::BadLineRequest => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/line/{id}")]
pub async fn get_line(
    identifier: Path<LineIdentifier>,
    database: Data<Database>,
) -> Result<Json<Line>, LineError> {
    let id: Result<i32, std::num::ParseIntError> = identifier.into_inner().id.parse::<i32>();
    if id.is_err() {
        return Err(LineError::BadLineRequest);
    }

    let line: Option<Line> = database
        .get_one::<Line>(
            sqlx::query_as::<_, Line>(format!("SELECT * FROM {} WHERE id=$1", Line::TABLE_NAME).as_str()).bind(id.unwrap())
        )
        .await;

    match line {
        Some(line) => Ok(Json(line)),
        None => Err(LineError::LineNotFound),
    }
}
