use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use orbit_types::models::emote::*;
use orbit_types::Snowflake;

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
		.route("/emotes/search", get(search_emotes))
}

async fn get_emote(Conn(conn): Conn, Path(id): Path<i64>) -> Result<Json<EmoteWithUser>> {
	let emote = conn
		.query_opt(
			r#"
			SELECT
				emotes.*,
				to_jsonb(users.*) AS "user",
				COALESCE(
					array_agg(
						versions.id ORDER BY versions.id
					) FILTER (
						WHERE versions.id IS NOT NULL
					),
					'{}'
				) AS versions
			FROM
				emotes
				LEFT JOIN users ON emotes.user_id = users.id
				LEFT JOIN versions ON emotes.id = versions.emote_id
			WHERE
				emotes.id = $1
			GROUP BY
				emotes.id,
				users.id
			"#,
			&[&id],
		)
		.await?
		.ok_or(JsonError::UnknownEntity("emote".into()))?
		.into();

	Ok(Json(emote))
}

async fn search_emotes(
	State(state): State<AppState>,
	Query(query): Query<SearchEmotesQuery>,
) -> Result<Json<Vec<Emote>>> {
	let filters: Vec<&str> = query.filters.split(',').collect();

	if let Some(query) = query.query {
		todo!();
	}

	Ok(Json(vec![]))
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

	// todo: image processing + s3

	Ok((StatusCode::CREATED, Json(emote)))
}

async fn update_emote(
	Conn(conn): Conn,
	Path(id): Path<i64>,
	Json(body): Json<UpdateEmote>,
) -> Result<Json<Emote>> {
	let emote = conn
		.query_opt(
			"
			UPDATE emotes
			SET
				approved = COALESCE($1, approved),
				nsfw = COALESCE($2, nsfw)
			WHERE id = $3
			RETURNING *
			",
			&[&body.approved, &body.nsfw, &id],
		)
		.await?
		.ok_or(JsonError::UnknownEntity("emote".into()))?
		.into();

	Ok(Json(emote))
}

async fn delete_emote(
	State(state): State<AppState>,
	Conn(conn): Conn,
	Path(id): Path<i64>,
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
		Err(JsonError::UnknownEntity("emote".into()).into())
	}
}
