use entity::sea_orm;
use poem::{error::ResponseError, http::StatusCode};

pub type Result<T, E = NeoiotError> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum NeoiotError {
    #[error("invalid topic :{0}")]
    InvalidTopic(String),
    #[error("http client request failed")]
    RequestClientError(#[from] reqwest::Error),
    #[error("emqx management api error:{0}")]
    EmqxManagementError(String),
    #[error("database error:{0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("specified {0} not found")]
    ObjectNotFound(String),
    #[error("feature not implemented yet")]
    NotImplemented,
}

impl ResponseError for NeoiotError {
    fn status(&self) -> StatusCode {
        match self {
            NeoiotError::InvalidTopic(_) => StatusCode::BAD_REQUEST,
            NeoiotError::RequestClientError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NeoiotError::EmqxManagementError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NeoiotError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            NeoiotError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            NeoiotError::ObjectNotFound(_) => StatusCode::BAD_REQUEST,
        }
    }
}
