use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use orbit_types::models::emote::*;
use orbit_types::snowflake::Snowflake;

use crate::auth::{self, AuthUser};
use crate::db::Conn;
use crate::error::{Error, JsonError};
use crate::{AppState, Result};

pub fn router(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/emotes", post(create_emote))
		.route("/emotes/:id", patch(update_emote))
		.route("/emotes/:id", delete(delete_emote))
		.route_layer(axum::middleware::from_fn_with_state(
			state.clone(),
			auth::middleware,
		))
		.route("/emotes/:id", get(get_emote))
}

async fn create_emote(
	Conn(conn): Conn,
	user: AuthUser,
	Json(body): Json<CreateEmote>,
) -> Result<(StatusCode, Json<EmoteWithUser>)> {
	let emote = conn
		.query_one(
			r#"
			INSERT INTO
				emotes (
					id, name, tags, width, height, public,
					animated, modifier, nsfw, user_id
				)
			VALUES
				($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
			RETURNING
				*,
				(
					SELECT to_jsonb(users.*) AS "user"
					FROM users
					WHERE users.id = $10
				)
			"#,
			&[
				&Snowflake::new().0,
				&body.name,
				&body.tags,
				&body.width,
				&body.height,
				&body.public,
				&body.animated,
				&body.modifier,
				&body.nsfw,
				&user.id,
			],
		)
		.await?
		.into();

	Ok((StatusCode::CREATED, Json(emote)))
}

async fn get_emote(Conn(conn): Conn, Path(id): Path<String>) -> Result<Json<EmoteWithUser>> {
	let emote = conn
		.query_opt(
			r#"
			SELECT
				emotes.*,
				to_jsonb(users.*) AS "user"
			FROM
				emotes
				LEFT JOIN users ON emotes.user_id = users.id
			WHERE emotes.id = $1
			"#,
			&[&id],
		)
		.await?
		.ok_or(JsonError::UnknownEmote)?
		.into();

	Ok(Json(emote))
}

async fn update_emote(
	Conn(conn): Conn,
	Path(id): Path<String>,
	Json(body): Json<UpdateEmote>,
) -> Result<Json<Emote>> {
	let emote = conn
		.query_opt(
			"
			UPDATE emotes
			SET
				approved = $1,
				nsfw = $2
			WHERE id = $3
			RETURNING *
			",
			&[&body.approved, &body.nsfw, &id],
		)
		.await?
		.ok_or(JsonError::UnknownEmote)?
		.into();

	Ok(Json(emote))
}

async fn delete_emote(
	State(state): State<AppState>,
	Conn(conn): Conn,
	Path(id): Path<String>,
) -> Result<StatusCode> {
	let deleted: bool = conn
		.query_one(
			"
			WITH returned AS (
				DELETE FROM emotes
				WHERE id = $1
				RETURNING 1
			)
			SELECT EXISTS (
				SELECT 1 FROM returned
			)
			",
			&[&id],
		)
		.await?
		.get(0);

	let mut to_delete: Vec<ObjectIdentifier> = vec![];
	let response = state
		.s3
		.list_objects_v2()
		.bucket("cdn.withorbit.xyz")
		.send()
		.await
		.map_err(|_| Error::Cdn)?;

	for object in response.contents.unwrap_or_default().iter() {
		if let Some(key) = &object.key {
			to_delete.push(
				ObjectIdentifier::builder()
					.set_key(Some(key.into()))
					.build()
					.map_err(|_| Error::Cdn)?,
			)
		}
	}

	state
		.s3
		.delete_objects()
		.bucket("cdn.withorbit.xyz")
		.delete(
			Delete::builder()
				.set_objects(Some(to_delete))
				.build()
				.map_err(|_| Error::Cdn)?,
		)
		.send()
		.await
		.map_err(|_| Error::Cdn)?;

	if deleted {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(JsonError::UnknownEmote.into())
	}
}
