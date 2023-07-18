use axum::{
    http::{header::LOCATION, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use thiserror::Error;

/// Our app's top level error type.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not Found: {0}")]
    NotFound(&'static str),
    #[error(" to `{0}` ")]
    InvalidLogin(String),
    #[error(" database error `{0}` ")]
    Database(String),
    #[error(" arg error `{0}` ")]
    InvalidArg(String),
    #[error("Invalid file format")]
    InvalidFileFormat,
    #[error("Error parsing `multipart/form-data` request.\n{0}")]
    MultipartError(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            AppError::InvalidLogin(str) => {
                let url = HeaderValue::try_from(str).expect("URI isn't a valid header value");
                (StatusCode::TEMPORARY_REDIRECT, [(LOCATION, url)]).into_response()
            }
            AppError::Database(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
            }

            AppError::InvalidArg(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
            }
            AppError::InvalidFileFormat => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
            }
            AppError::MultipartError(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
            }
            AppError::Anyhow(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
        }
    }
}

// Any errors from sqlx get converted to CustomError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> AppError {
        AppError::Database(err.to_string())
    }
}

impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> AppError {
        AppError::InvalidArg(err.to_string())
    }
}
