use axum::extract::{FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::{header, HeaderMap, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use axum::RequestPartsExt;

use crate::db::{Conn, Connection};
use crate::error::{Error, JsonError};
use crate::{AppState, Result};

pub async fn middleware(Conn(conn): Conn, req: Request, next: Next) -> Result<Response> {
	verify_token(req.headers(), &conn).await?;

	Ok(next.run(req).await)
}

pub type AuthUser = orbit_types::models::user::User;

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
	type Rejection = Error;

	async fn from_request_parts(
		parts: &mut Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		let Conn(conn) = parts.extract_with_state::<Conn, AppState>(state).await?;

		let user_id = verify_token(&parts.headers, &conn).await?;

		let user = conn
			.query_opt("SELECT * FROM users WHERE id = $1", &[&user_id])
			.await?
			.ok_or(JsonError::UnknownUser)?
			.into();

		Ok(user)
	}
}

async fn verify_token(headers: &HeaderMap<HeaderValue>, conn: &Connection) -> Result<String> {
	let token = headers
		.get(header::AUTHORIZATION)
		.and_then(|value| value.to_str().ok())
		.and_then(|value| {
			if value.starts_with("Bearer") {
				Some(value.trim_start_matches("Bearer ").to_string())
			} else {
				None
			}
		})
		.ok_or(JsonError::Unauthorized)?;

	let user_id = conn
		.query_opt("SELECT user_id FROM sessions WHERE id = $1", &[&token])
		.await?
		.ok_or(JsonError::InvalidToken)?
		.get(0);

	Ok(user_id)
}
