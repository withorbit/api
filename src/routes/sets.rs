use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use serde::{Deserialize, Serialize};

use crate::snowflake::Snowflake;
use crate::{AppState, Error, Result};

use super::emotes::Emote;

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
	pub emotes: Vec<Emote>,
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/sets", post(create_set))
		.route("/sets/:id", get(get_set))
		.route("/sets/:id", patch(update_set))
		.route("/sets/:id", delete(delete_set))
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
	let raw_set = sqlx::query!(
		r#"
			SELECT
				sets.*,
				set_emotes.coalesce AS emotes
			FROM
				sets
				LEFT JOIN LATERAL (
					SELECT COALESCE(jsonb_agg(data), '[]')
					FROM (
						SELECT get_emotes.data
						FROM
							emote_in_sets AS m2m
							LEFT JOIN LATERAL (
								SELECT to_jsonb(emote) AS data
								FROM (
									SELECT emotes.*
									FROM emotes
									WHERE m2m.emote_id = emotes.id
								) AS emote
							) AS get_emotes ON true
						WHERE m2m.set_id = sets.id
					) AS _
				) AS set_emotes ON TRUE
			WHERE sets.user_id = $1
		"#,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(Error::NotFound)?;

	let emotes: Vec<Emote> = serde_json::from_value(raw_set.emotes.unwrap_or_default())?;

	Ok(Json(EmoteSetWithEmotes {
		id: raw_set.id,
		name: raw_set.name,
		capacity: raw_set.capacity,
		user_id: raw_set.user_id,
		parent_id: raw_set.parent_id,
		emotes,
	}))
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
	.ok_or(Error::NotFound)?;

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
			)
		"#,
		id
	)
	.fetch_one(&state.pool)
	.await?;

	if deleted.unwrap_or_default() {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(Error::NotFound)
	}
}
