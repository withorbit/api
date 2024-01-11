use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::types::Json as Jsonb;

use crate::error::{JsonError, ResultExt};
use crate::snowflake::Snowflake;
use crate::{AppState, Result};

use super::emotes::Emote;

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/sets", post(create_set))
		.route("/sets/:id", get(get_set))
		.route("/sets/:id", patch(update_set))
		.route("/sets/:id", delete(delete_set))
		.route("/sets/:id/emotes/:emoteId", put(add_set_emote))
		.route("/sets/:id/emotes/:emoteId", delete(remove_set_emote))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmoteSet {
	pub id: String,
	pub name: String,
	pub capacity: i32,
	pub user_id: String,
	pub parent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmoteSetWithEmotes {
	pub id: String,
	pub name: String,
	pub capacity: i32,
	pub user_id: String,
	pub parent_id: Option<String>,
	pub emotes: Jsonb<Vec<Emote>>,
}

#[derive(Debug, Deserialize)]
struct CreateEmoteSet {
	name: String,
	capacity: i32,
}

async fn create_set(
	State(state): State<AppState>,
	Json(body): Json<CreateEmoteSet>,
) -> Result<Json<EmoteSet>> {
	// todo: handle auth

	let set = sqlx::query_as!(
		EmoteSet,
		r#"
			INSERT INTO sets (id, name, capacity, user_id)
			VALUES ($1, $2, $3, $4)
			RETURNING *
		"#,
		Snowflake::new().0,
		body.name,
		body.capacity,
		"!!TODO!!"
	)
	.fetch_one(&state.pool)
	.await?;

	Ok(Json(set))
}

async fn get_set(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<EmoteSetWithEmotes>> {
	let set = sqlx::query_as!(
		EmoteSetWithEmotes,
		r#"
			SELECT
				sets.*,
				COALESCE(
					jsonb_agg(emotes.*) FILTER (WHERE emotes.id IS NOT NULL), '[]'
				) AS "emotes!: _"
			FROM
				sets
				LEFT JOIN emotes_to_sets AS m2m ON sets.id = m2m.set_id
				LEFT JOIN emotes ON m2m.emote_id = emotes.id
			WHERE sets.id = $1
			GROUP BY sets.id
		"#,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(JsonError::UnknownEmoteSet)?;

	Ok(Json(set))
}

#[derive(Debug, Deserialize)]
struct UpdateEmoteSet {
	name: Option<String>,
	capacity: Option<i32>,
}

async fn update_set(
	State(state): State<AppState>,
	Path(id): Path<String>,
	Json(body): Json<UpdateEmoteSet>,
) -> Result<Json<EmoteSet>> {
	let set = sqlx::query_as!(
		EmoteSet,
		r#"
			UPDATE sets
			SET
				name = $1,
				capacity = $2
			WHERE id = $3
			RETURNING *
		"#,
		body.name,
		body.capacity,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(JsonError::UnknownEmoteSet)?;

	Ok(Json(set))
}

async fn delete_set(State(state): State<AppState>, Path(id): Path<String>) -> Result<StatusCode> {
	let deleted = sqlx::query_scalar!(
		r#"
			WITH returned AS (
				DELETE FROM sets
				WHERE id = $1
				RETURNING 1
			)
			SELECT EXISTS (
				SELECT 1 FROM returned
			) AS "exists!"
		"#,
		id
	)
	.fetch_one(&state.pool)
	.await?;

	if deleted {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(JsonError::UnknownEmoteSet.into())
	}
}

async fn add_set_emote(
	State(state): State<AppState>,
	Path((set_id, emote_id)): Path<(String, String)>,
) -> Result<StatusCode> {
	sqlx::query!(
		"INSERT INTO emotes_to_sets (set_id, emote_id)
		VALUES ($1, $2)
		ON CONFLICT DO NOTHING",
		set_id,
		emote_id
	)
	.execute(&state.pool)
	.await
	.on_constraint(
		"emotes_to_sets_set_id_fkey",
		JsonError::UnknownEmoteSet.into(),
	)
	.on_constraint(
		"emotes_to_sets_emote_id_fkey",
		JsonError::UnknownEmote.into(),
	)?;

	Ok(StatusCode::NO_CONTENT)
}

async fn remove_set_emote(
	State(state): State<AppState>,
	Path((set_id, emote_id)): Path<(String, String)>,
) -> Result<StatusCode> {
	let exists = sqlx::query_scalar!("SELECT id FROM sets WHERE id = $1", set_id)
		.fetch_optional(&state.pool)
		.await?;

	if exists.is_none() {
		return Err(JsonError::UnknownEmoteSet.into());
	}

	let deleted = sqlx::query_scalar!(
		r#"
			WITH returned AS (
				DELETE FROM emotes_to_sets
				WHERE set_id = $1 AND emote_id = $2
				RETURNING 1
			)
			SELECT EXISTS (
				SELECT 1 FROM returned
			) AS "exists!"
		"#,
		set_id,
		emote_id
	)
	.fetch_one(&state.pool)
	.await?;

	if deleted {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(JsonError::UnknownEmote.into())
	}
}
