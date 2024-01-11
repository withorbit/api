use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("{0}")]
	BadRequest(String),
	#[error("{0}")]
	NotFound(String),
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
	fn status_code(self) -> StatusCode {
		use self::Error::*;

		match self {
			BadRequest(_) => StatusCode::BAD_REQUEST,
			NotFound(_) => StatusCode::NOT_FOUND,
			UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
			Cdn | Database(_) | Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

		let body = json!({ "message": self.to_string() });

		(self.status_code(), Json(body)).into_response()
	}
}

impl From<JsonError> for Error {
	fn from(value: JsonError) -> Self {
		let (code, message) = value.to_pair();

		match code {
			400 => Error::BadRequest(message),
			404 => Error::NotFound(message),
			_ => unreachable!(),
		}
	}
}

pub enum JsonError {
	UnknownUser,
	UnknownEmote,
	UnknownEmoteSet,
	UserCannotAddSelf,
}

impl JsonError {
	fn to_pair(self) -> (u32, String) {
		use self::JsonError::*;

		let pair = match self {
			UnknownUser => (404, "Unknown user."),
			UnknownEmote => (404, "Unknown emote."),
			UnknownEmoteSet => (404, "Unknown emote set."),
			UserCannotAddSelf => (
				400,
				"User cannot add themselves as an editor to their own channel.",
			),
		};

		(pair.0, pair.1.to_string())
	}
}

pub trait ResultExt<T> {
	fn on_constraint(self, name: &str, e: Error) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
	E: Into<Error>,
{
	fn on_constraint(self, name: &str, e: Error) -> Result<T, Error> {
		self.map_err(|err| match err.into() {
			Error::Database(sqlx::Error::Database(err)) if err.constraint() == Some(name) => e,
			err => err,
		})
	}
}
