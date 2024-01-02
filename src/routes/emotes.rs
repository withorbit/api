use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use sqlx::types::Json as Jsonb;

use crate::error::Error;
use crate::snowflake::Snowflake;
use crate::{AppState, Result};

use super::users::User;

#[derive(Debug, Deserialize, Serialize)]
pub struct Emote {
	pub id: String,
	pub name: String,
	pub tags: Vec<String>,
	pub width: i32,
	pub height: i32,
	pub approved: bool,
	pub public: bool,
	pub animated: bool,
	pub modifier: bool,
	pub nsfw: bool,

	#[serde(skip_serializing)]
	pub user_id: String,
	pub user: Jsonb<User>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEmote {
	name: String,
	tags: Vec<String>,
	width: i32,
	height: i32,
	public: bool,
	animated: bool,
	modifier: bool,
	nsfw: bool,
	user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmote {
	approved: Option<bool>,
	nsfw: Option<bool>,
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/emotes", post(create_emote))
		.route("/emotes/:id", get(get_emote))
		.route("/emotes/:id", patch(update_emote))
		.route("/emotes/:id", delete(delete_emote))
}

async fn create_emote(
	State(state): State<AppState>,
	Json(body): Json<CreateEmote>,
) -> Result<(StatusCode, Json<Emote>)> {
	tracing::debug!(?body);

	let emote = sqlx::query_as!(
		Emote,
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
					SELECT to_jsonb("user")
					FROM (SELECT * FROM users)
					AS "user"
				) AS "user!: _"
		"#,
		Snowflake::new().0,
		body.name,
		&body.tags,
		body.width,
		body.height,
		body.public,
		body.animated,
		body.modifier,
		body.nsfw,
		body.user_id
	)
	.fetch_one(&state.pool)
	.await?;

	Ok((StatusCode::CREATED, Json(emote)))
}

async fn get_emote(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<Emote>> {
	let emote = sqlx::query_as!(
		Emote,
		r#"
			SELECT
				*,
				(
					SELECT to_jsonb("user")
					FROM (SELECT * FROM users)
					AS "user"
				) AS "user!: _"
			FROM emotes
			WHERE id = $1
		"#,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(Error::NotFound)?;

	Ok(Json(emote))
}

async fn update_emote(
	State(state): State<AppState>,
	Path(id): Path<String>,
	Json(body): Json<UpdateEmote>,
) -> Result<Json<Emote>> {
	let emote = sqlx::query_as!(
		Emote,
		r#"
			UPDATE emotes
			SET
				approved = $1,
				nsfw = $2
			WHERE id = $3
			RETURNING
				*,
				(
					SELECT to_jsonb("user")
					FROM (SELECT * FROM users)
					AS "user"
				) AS "user!: _"
		"#,
		body.approved,
		body.nsfw,
		id
	)
	.fetch_optional(&state.pool)
	.await?
	.ok_or(Error::NotFound)?;

	Ok(Json(emote))
}

async fn delete_emote(State(state): State<AppState>, Path(id): Path<String>) -> Result<StatusCode> {
	let result = sqlx::query!(
		r#"
			WITH returned AS (
				DELETE FROM emotes
				WHERE id = $1
				RETURNING 1
			)
			SELECT EXISTS (
				SELECT 1 FROM returned
			) AS "deleted!"
		"#,
		id
	)
	.fetch_one(&state.pool)
	.await?;

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

	if result.deleted {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(Error::NotFound)
	}
}
