use poem::{error::ResponseError, http::StatusCode};

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct CustomError {
    message: String,
}

impl CustomError {}

impl ResponseError for CustomError {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
