use axum::extract::{Json, Path, State};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::{AppState, Error, Result};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/users/me", get(get_current_user))
		.route("/users/:id", get(get_user))
		.route("/users/:id/emotes", get(get_user_emotes))
		.route("/users/:id/sets", get(get_user_sets))
}

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
	.ok_or(Error::NotFound("Unknown user.".to_string()))?;

	Ok(Json(user))
}

#[derive(Deserialize, Serialize)]
struct UserEmote {
	id: String,
	name: String,
	tags: Vec<String>,
	width: i32,
	height: i32,
	approved: bool,
	public: bool,
	animated: bool,
	modifier: bool,
	nsfw: bool,
	user_id: String,
}

async fn get_user_emotes(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<Vec<UserEmote>>> {
	if !user_exists(&state.pool, &id).await {
		return Err(Error::NotFound("Unknown user.".to_string()));
	}

	let emotes = sqlx::query_as!(
		UserEmote,
		"
			SELECT emotes.*
			FROM users
				LEFT JOIN emotes ON true
			WHERE user_id = $1
		",
		id
	)
	.fetch_all(&state.pool)
	.await?;

	Ok(Json(emotes))
}

#[derive(Deserialize, Serialize)]
struct UserEmoteSet {
	id: String,
	name: String,
	capacity: i32,
	user_id: String,
	parent_id: Option<String>,
}

async fn get_user_sets(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<Vec<UserEmoteSet>>> {
	if !user_exists(&state.pool, &id).await {
		return Err(Error::NotFound("Unknown user.".to_string()));
	}

	let sets = sqlx::query_as!(
		UserEmoteSet,
		"
			SELECT sets.*
			FROM users
				LEFT JOIN sets ON true
			WHERE user_id = $1
		",
		id
	)
	.fetch_all(&state.pool)
	.await?;

	Ok(Json(sets))
}

async fn user_exists(pool: &Pool<Postgres>, id: &String) -> bool {
	sqlx::query_scalar!(
		r#"SELECT EXISTS (SELECT id FROM users WHERE id = $1) AS "exists!""#,
		id
	)
	.fetch_one(pool)
	.await
	.unwrap()
}
