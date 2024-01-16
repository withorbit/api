use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{header, HeaderMap, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use sqlx::{Pool, Postgres};

use crate::error::JsonError;
use crate::routes::users::User;
use crate::{AppState, Error, Result};

pub async fn middleware(
	State(state): State<AppState>,
	req: Request,
	next: Next,
) -> Result<Response> {
	verify_token(req.headers(), &state.pool).await?;

	Ok(next.run(req).await)
}

pub type AuthUser = User;

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
	type Rejection = Error;

	async fn from_request_parts(
		parts: &mut Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		let user_id = verify_token(&parts.headers, &state.pool).await?;

		let user = sqlx::query_as!(
			User,
			r#"
				SELECT
					id,
					twitch_id,
					username,
					avatar_url,
					roles AS "roles: _",
					badge_url,
					color_id,
					channel_set_id
				FROM users
				WHERE id = $1
			"#,
			user_id
		)
		.fetch_optional(&state.pool)
		.await?
		.ok_or(JsonError::UnknownUser)?;

		Ok(user)
	}
}

async fn verify_token(headers: &HeaderMap<HeaderValue>, pool: &Pool<Postgres>) -> Result<String> {
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

	let user_id = sqlx::query_scalar!("SELECT user_id FROM sessions WHERE id = $1", token)
		.fetch_optional(pool)
		.await?
		.ok_or(JsonError::InvalidToken)?;

	Ok(user_id)
}
