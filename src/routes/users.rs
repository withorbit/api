use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, put};
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};
use sqlx::{Pool, Postgres};

use crate::error::{JsonError, ResultExt};
use crate::{AppState, Error, Result};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/users/@me", get(get_current_user))
		.route("/users/@me/editors/:id", put(add_user_editor))
		.route("/users/@me/editors/:id", delete(remove_user_editor))
		.route("/users/:id", get(get_user))
		.route("/users/:id/editors", get(get_user_editors))
		.route("/users/:id/emotes", get(get_user_emotes))
		.route("/users/:id/sets", get(get_user_sets))
		.route("/users/:id/sets/@channel", get(get_user_channel_set))
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

impl PgHasArrayType for Role {
	fn array_type_info() -> sqlx::postgres::PgTypeInfo {
		PgTypeInfo::with_name("_role")
	}
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
	pub channel_set_id: String,
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
				roles AS "roles: _",
				badge_url,
				color_id,
				channel_set_id
			FROM users
			WHERE id = $1
		"#,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(JsonError::UnknownUser)?;

	Ok(Json(user))
}

async fn get_user_editors(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<Vec<User>>> {
	let editors = sqlx::query_as!(
		User,
		r#"
			SELECT
				editor.id,
				editor.twitch_id,
				editor.username,
				editor.avatar_url,
				editor.roles AS "roles: _",
				editor.badge_url,
				editor.color_id,
				editor.channel_set_id
			FROM
				users
				JOIN users_to_editors AS m2m ON users.id = m2m.user_id
				JOIN users AS editor ON editor.id = m2m.editor_id
			WHERE users.id = $1;
		"#,
		id
	)
	.fetch_all(&state.pool)
	.await?;

	Ok(Json(editors))
}

async fn add_user_editor(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<StatusCode> {
	sqlx::query!(
		"INSERT INTO users_to_editors (user_id, editor_id)
		VALUES ($1, $2)
		ON CONFLICT DO NOTHING",
		"!!TODO!!",
		id
	)
	.execute(&state.pool)
	.await
	.on_constraint("user_cannot_add_self", JsonError::UserCannotAddSelf.into())?;

	Ok(StatusCode::NO_CONTENT)
}

async fn remove_user_editor(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<StatusCode> {
	let deleted = sqlx::query_scalar!(
		r#"
			WITH returned AS (
				DELETE FROM users_to_editors
				WHERE user_id = $1 AND editor_id = $2
				RETURNING 1
			)
			SELECT EXISTS (
				SELECT 1 FROM returned
			) AS "exists!"
		"#,
		"!!TODO!!",
		id
	)
	.fetch_one(&state.pool)
	.await?;

	if deleted {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(JsonError::UnknownUser.into())
	}
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
			FROM
				users
				JOIN emotes ON users.id = emotes.user_id
			WHERE user_id = $1
			ORDER BY id
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
		return Err(JsonError::UnknownUser.into());
	}

	let sets = sqlx::query_as!(
		UserEmoteSet,
		"
			SELECT sets.*
			FROM
				users
				JOIN sets ON users.id = sets.user_id
			WHERE user_id = $1
			ORDER BY id
		",
		id
	)
	.fetch_all(&state.pool)
	.await?;

	Ok(Json(sets))
}

async fn get_user_channel_set(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<UserEmoteSet>> {
	let set = sqlx::query_as!(
		UserEmoteSet,
		"
			SELECT sets.*
			FROM
				users
				JOIN sets ON users.channel_set_id = sets.id
			WHERE users.id = $1
		",
		id
	)
	.fetch_one(&state.pool)
	.await?;

	Ok(Json(set))
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
