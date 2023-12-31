use axum::extract::{Json, Path, State};
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};

use crate::{AppState, Error, Result};

use super::emotes::Emote;
use super::sets::EmoteSet;

#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "role", rename_all = "lowercase")]
pub enum Role {
	Verified,
	Subscriber,
	Founder,
	Contributor,
	Maintainer,
	Moderator,
	Admin,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
	pub id: String,
	pub twitch_id: i32,
	pub username: String,
	pub avatar_url: String,
	pub roles: Vec<Role>,
	pub badge_url: Option<String>,
	pub color_id: Option<String>,
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/users/me", get(get_current_user))
		.route("/users/:id", get(get_user))
		.route("/users/:id/emotes", get(get_user_emotes))
		.route("/users/:id/sets", get(get_user_sets))
}

async fn get_current_user() {
	todo!()
}

async fn get_user(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<User>> {
	let user = sqlx::query_as!(
		User,
		r#"
			SELECT
				id,
				twitch_id,
				username,
				avatar_url,
				roles AS "roles: Vec<Role>",
				badge_url,
				color_id
			FROM users
			WHERE id = $1
		"#,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(Error::NotFound)?;

	Ok(Json(user))
}

async fn get_user_emotes(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<Vec<Emote>>> {
	todo!()
}

async fn get_user_sets(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<Vec<EmoteSet>>> {
	todo!()
}
