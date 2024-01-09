use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::Router;
use serde::Serialize;
use sqlx::types::Json as Jsonb;

use crate::error::Error;
use crate::{AppState, Result};

use super::emotes::Emote;
use super::users::User;

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/channels/:id", get(get_channel))
		.route("/channels/:id/editors", get(get_channel_editors))
		.route("/channels/:id/editors/:userId", put(add_channel_editor))
		.route("/channels/:id/emotes", get(get_channel_emotes))
}

#[derive(Debug, Serialize)]
pub struct Channel {
	pub id: String,
	pub user: Jsonb<User>,
}

async fn get_channel(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<Channel>> {
	let channel = sqlx::query_as!(
		Channel,
		r#"
			SELECT
				id,
				channel_user."user" AS "user!: _"
			FROM
				channels
				LEFT JOIN LATERAL (
					SELECT to_jsonb(data) AS "user"
					FROM (
						SELECT users.*
						FROM users
						WHERE user_id = users.id
					) AS data
				) AS channel_user ON TRUE
			WHERE id = $1
		"#,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(Error::NotFound)?;

	Ok(Json(channel))
}

async fn get_channel_editors(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<Vec<User>>> {
	let Jsonb(editors) = sqlx::query_scalar!(
		r#"
			SELECT
				channel_editors.coalesce AS "editors!: Jsonb<Vec<User>>"
			FROM
				channels
				LEFT JOIN LATERAL (
					SELECT COALESCE(jsonb_agg(data), '[]')
					FROM (
						SELECT get_editors.data
						FROM
							editor_for AS m2m
							LEFT JOIN LATERAL (
								SELECT to_jsonb(editor) AS data
								FROM (
									SELECT users.*
									FROM users
									WHERE m2m.user_id = users.id
								) AS editor
							) AS get_editors ON true
						WHERE m2m.channel_id = channels.id
					) AS _
				) AS channel_editors ON TRUE
			WHERE channels.id = $1
		"#,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(Error::NotFound)?;

	Ok(Json(editors))
}

async fn add_channel_editor(
	State(state): State<AppState>,
	Path((id, user_id)): Path<(String, String)>,
) -> Result<StatusCode> {
	let returned = sqlx::query_scalar!(
		r#"
			WITH returned AS (
				UPDATE editor_for
				SET
					channel_id = $1,
					user_id = $2
				RETURNING 1
			)
			SELECT EXISTS (
				SELECT 1 FROM returned
			)
		"#,
		id,
		user_id
	)
	.fetch_one(&state.pool)
	.await?;

	if returned.is_some() {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(Error::NotFound)
	}
}

async fn get_channel_emotes(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> Result<Json<Vec<Emote>>> {
	let Jsonb(emotes) = sqlx::query_scalar!(
		r#"
			SELECT
				_emotes.coalesce AS "emotes!: Jsonb<Vec<Emote>>"
			FROM
				channels
				LEFT JOIN LATERAL (
					SELECT COALESCE(jsonb_agg(data), '[]')
					FROM (
						SELECT get_emotes.data
						FROM
							channel_emotes AS m2m
							LEFT JOIN LATERAL (
								SELECT to_jsonb(emote) AS data
								FROM (
									SELECT emotes.*
									FROM emotes
									WHERE m2m.emote_id = emotes.id
								) AS emote
							) AS get_emotes ON true
						WHERE m2m.channel_id = channels.id
					) AS _
				) AS _emotes ON TRUE
			WHERE channels.id = $1
		"#,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(Error::NotFound)?;

	Ok(Json(emotes))
}
