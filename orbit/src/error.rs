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

	#[error("{0}")]
	Unauthorized(String),

	#[error("{0}")]
	Forbidden(String),

	#[error("{0}")]
	Conflict(String),

	#[error("422 Unprocessable Entity")]
	UnprocessableEntity,

	#[error("500 Internal Server Error (CDN)")]
	Cdn,

	#[error("500 Internal Server Error (Database)")]
	Database(#[from] tokio_postgres::Error),

	#[error("500 Internal Server Error (JSON)")]
	Json(#[from] serde_json::Error),

	#[error("500 Internal Server Error")]
	Generic,
}

impl Error {
	fn status_code(self) -> StatusCode {
		use self::Error::*;

		match self {
			BadRequest(_) => StatusCode::BAD_REQUEST,
			NotFound(_) => StatusCode::NOT_FOUND,
			Unauthorized(_) => StatusCode::UNAUTHORIZED,
			Forbidden(_) => StatusCode::FORBIDDEN,
			Conflict(_) => StatusCode::CONFLICT,
			UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
			Cdn | Database(_) | Json(_) | Generic => StatusCode::INTERNAL_SERVER_ERROR,
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
		use self::Error::*;

		let message = value.to_string();

		match value.status_code() {
			400 => BadRequest(message),
			401 => Unauthorized(message),
			403 => Forbidden(message),
			404 => NotFound(message),
			409 => Conflict(message),
			_ => unreachable!(),
		}
	}
}

#[derive(thiserror::Error, Debug)]
pub enum JsonError {
	#[error("Unknown user.")]
	UnknownUser,

	#[error("Unknown emote.")]
	UnknownEmote,

	#[error("Unknown emote set.")]
	UnknownEmoteSet,

	#[error("User cannot add themselves as an editor to their own channel.")]
	UserCannotAddSelf,

	#[error("Unauthorized.")]
	Unauthorized,

	#[error("Invalid bearer token.")]
	InvalidToken,

	#[error("Forbidden.")]
	Forbidden,

	#[error("Color already exists.")]
	ColorExists,
}

impl JsonError {
	fn status_code(self) -> u32 {
		use self::JsonError::*;

		match self {
			UserCannotAddSelf => 400,
			Unauthorized | InvalidToken => 401,
			Forbidden => 403,
			UnknownUser | UnknownEmote | UnknownEmoteSet => 404,
			ColorExists => 409,
		}
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
			Error::Database(err)
				if matches!(
					err.as_db_error().and_then(|e| e.constraint()),
					Some(constraint) if constraint == name
				) =>
			{
				e
			}
			err => err,
		})
	}
}
