use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("404 Not Found")]
	NotFound,
	#[error("422 Unprocessable Entity")]
	UnprocessableEntity,
	#[error("500 Internal Server Error (CDN)")]
	Cdn,
	#[error("500 Internal Server Error (Database)")]
	Database(#[from] sqlx::Error),
	#[error("500 Internal Server Error (JSON)")]
	Json(#[from] serde_json::Error),
}

impl Error {
	fn status_code(&self) -> StatusCode {
		match self {
			Self::NotFound => StatusCode::NOT_FOUND,
			Self::UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
			Self::Cdn | Self::Database(_) | Self::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}
}

impl IntoResponse for Error {
	fn into_response(self) -> Response {
		match self {
			Self::Database(ref err) => {
				tracing::error!(?err);
			}
			Self::Json(ref err) => {
				tracing::error!(?err);
			}
			_ => (),
		}

		(self.status_code(), self.to_string()).into_response()
	}
}
