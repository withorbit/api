use axum::extract::{Json, Path};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use orbit_types::models::set::*;
use orbit_types::snowflake::Snowflake;

use crate::auth::{self, AuthUser};
use crate::db::Conn;
use crate::error::{JsonError, ResultExt};
use crate::{AppState, Result};

pub fn router(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/sets", post(create_set))
		.route("/sets/:id", patch(update_set))
		.route("/sets/:id", delete(delete_set))
		.route("/sets/:id/emotes/:emoteId", put(add_set_emote))
		.route("/sets/:id/emotes/:emoteId", delete(remove_set_emote))
		.route_layer(axum::middleware::from_fn_with_state(
			state.clone(),
			auth::middleware,
		))
		.route("/sets/:id", get(get_set))
}

async fn create_set(
	Conn(conn): Conn,
	user: AuthUser,
	Json(body): Json<CreateEmoteSet>,
) -> Result<Json<EmoteSet>> {
	let set = conn
		.query_one(
			"
			INSERT INTO sets (id, name, capacity, user_id)
			VALUES ($1, $2, $3, $4)
			RETURNING *
			",
			&[&Snowflake::new().0, &body.name, &body.capacity, &user.id],
		)
		.await?
		.into();

	Ok(Json(set))
}

async fn get_set(Conn(conn): Conn, Path(id): Path<i64>) -> Result<Json<EmoteSetWithEmotes>> {
	let set = conn
		.query_opt(
			"
			SELECT
				sets.*,
				COALESCE(
					jsonb_agg(emotes.*) FILTER (WHERE emotes.id IS NOT NULL), '[]'
				) AS emotes
			FROM
				sets
				LEFT JOIN emotes_to_sets AS m2m ON sets.id = m2m.set_id
				LEFT JOIN emotes ON m2m.emote_id = emotes.id
			WHERE sets.id = $1
			GROUP BY sets.id
			",
			&[&id],
		)
		.await?
		.ok_or(JsonError::UnknownEmoteSet)?
		.into();

	Ok(Json(set))
}

async fn update_set(
	Conn(conn): Conn,
	Path(id): Path<i64>,
	Json(body): Json<UpdateEmoteSet>,
) -> Result<Json<EmoteSet>> {
	let set = conn
		.query_opt(
			"
			UPDATE sets
			SET
				name = $1,
				capacity = $2
			WHERE id = $3
			RETURNING *
			",
			&[&body.name, &body.capacity, &id],
		)
		.await?
		.ok_or(JsonError::UnknownEmoteSet)?
		.into();

	Ok(Json(set))
}

async fn delete_set(Conn(conn): Conn, user: AuthUser, Path(id): Path<i64>) -> Result<StatusCode> {
	let deleted = conn
		.query_one(
			"
			WITH returned AS (
				DELETE FROM sets
				WHERE id = $1 AND user_id = $2
				RETURNING 1
			)
			SELECT EXISTS (
				SELECT 1 FROM returned
			)
			",
			&[&id, &user.id],
		)
		.await?
		.get(0);

	if deleted {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(JsonError::UnknownEmoteSet.into())
	}
}

async fn add_set_emote(
	Conn(conn): Conn,
	Path((set_id, emote_id)): Path<(i64, i64)>,
) -> Result<StatusCode> {
	conn.execute(
		"
		INSERT INTO emotes_to_sets (set_id, emote_id)
		VALUES ($1, $2)
		ON CONFLICT DO NOTHING
		",
		&[&set_id, &emote_id],
	)
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
	Conn(conn): Conn,
	Path((set_id, emote_id)): Path<(i64, i64)>,
) -> Result<StatusCode> {
	let exists = conn
		.query_opt("SELECT id FROM sets WHERE id = $1", &[&set_id])
		.await?;

	if exists.is_none() {
		return Err(JsonError::UnknownEmoteSet.into());
	}

	let deleted = conn
		.query_one(
			"
			WITH returned AS (
				DELETE FROM emotes_to_sets
				WHERE set_id = $1 AND emote_id = $2
				RETURNING 1
			)
			SELECT EXISTS (
				SELECT 1 FROM returned
			)
			",
			&[&set_id, &emote_id],
		)
		.await?
		.get(0);

	if deleted {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(JsonError::UnknownEmote.into())
	}
}
