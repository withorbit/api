use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, put};
use axum::{debug_handler, Router};
use orbit_derive::FromRow;
use postgres_types::{ToSql, Type};
use serde::{Deserialize, Serialize};
use tokio_postgres::types::FromSql;

use crate::auth::{self, AuthUser};
use crate::db::{Conn, Connection};
use crate::error::{Error, JsonError, ResultExt};
use crate::{AppState, Result};

pub fn router(state: &AppState) -> Router<AppState> {
	Router::new()
		.route("/users/@me", get(get_current_user))
		.route("/users/@me/editors/:id", put(add_user_editor))
		.route("/users/@me/editors/:id", delete(remove_user_editor))
		.route_layer(axum::middleware::from_fn_with_state(
			state.clone(),
			auth::middleware,
		))
		.route("/users/:id", get(get_user))
		.route("/users/:id/editors", get(get_user_editors))
		.route("/users/:id/emotes", get(get_user_emotes))
		.route("/users/:id/sets", get(get_user_sets))
		.route("/users/:id/sets/@channel", get(get_user_channel_set))
}

#[derive(Debug, Deserialize, Serialize, ToSql, FromSql)]
#[postgres(name = "role", rename_all = "lowercase")]
pub enum Role {
	Verified,
	Subscriber,
	Founder,
	Contributor,
	Maintainer,
	Moderator,
	Admin,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
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

impl FromSql<'_> for User {
	fn from_sql(
		_: &Type,
		raw: &'_ [u8],
	) -> std::result::Result<Self, Box<dyn std::error::Error + Sync + Send>> {
		Ok(serde_json::from_slice(&raw[1..])?)
	}

	fn accepts(ty: &Type) -> bool {
		ty == &Type::JSONB
	}
}

async fn get_current_user(user: AuthUser) -> Result<Json<User>> {
	Ok(Json(user))
}

#[debug_handler]
async fn get_user(
	_: State<AppState>,
	Conn(conn): Conn,
	Path(id): Path<String>,
) -> Result<Json<User>> {
	let user = conn
		.query_opt("SELECT * FROM users WHERE id = $1", &[&id])
		.await?
		.ok_or(JsonError::UnknownUser)?
		.into();

	Ok(Json(user))
}

async fn get_user_editors(
	_: State<AppState>,
	Conn(conn): Conn,
	Path(id): Path<String>,
) -> Result<Json<Vec<User>>> {
	let editors = conn
		.query(
			"
			SELECT
				editor.*
			FROM
				users
				JOIN users_to_editors AS m2m ON users.id = m2m.user_id
				JOIN users AS editor ON editor.id = m2m.editor_id
			WHERE users.id = $1
			",
			&[&id],
		)
		.await?
		.into_iter()
		.map(|row| row.into())
		.collect();

	Ok(Json(editors))
}

async fn add_user_editor(
	_: State<AppState>,
	Conn(conn): Conn,
	user: AuthUser,
	Path(id): Path<String>,
) -> Result<StatusCode> {
	conn.execute(
		"
		INSERT INTO users_to_editors (user_id, editor_id)
		VALUES ($1, $2)
		ON CONFLICT DO NOTHING
		",
		&[&user.id, &id],
	)
	.await
	.on_constraint("user_cannot_add_self", JsonError::UserCannotAddSelf.into())?;

	Ok(StatusCode::NO_CONTENT)
}

async fn remove_user_editor(
	_: State<AppState>,
	Conn(conn): Conn,
	user: AuthUser,
	Path(id): Path<String>,
) -> Result<StatusCode> {
	let deleted = conn
		.query_one(
			"
			WITH returned AS (
				DELETE FROM users_to_editors
				WHERE user_id = $1 AND editor_id = $2
				RETURNING 1
			)
			SELECT EXISTS (
				SELECT 1 FROM returned
			)
			",
			&[&user.id, &id],
		)
		.await?
		.get(0);

	if deleted {
		Ok(StatusCode::NO_CONTENT)
	} else {
		Err(JsonError::UnknownUser.into())
	}
}

#[derive(Deserialize, Serialize, FromRow)]
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
	_: State<AppState>,
	Conn(conn): Conn,
	Path(id): Path<String>,
) -> Result<Json<Vec<UserEmote>>> {
	if !user_exists(&conn, &id).await {
		return Err(Error::NotFound("Unknown user.".to_string()));
	}

	let emotes = conn
		.query(
			"
			SELECT emotes.*
			FROM
				users
				JOIN emotes ON users.id = emotes.user_id
			WHERE user_id = $1
			ORDER BY id
			",
			&[&id],
		)
		.await?
		.into_iter()
		.map(|row| row.into())
		.collect();

	Ok(Json(emotes))
}

#[derive(Deserialize, Serialize, FromRow)]
struct UserEmoteSet {
	id: String,
	name: String,
	capacity: i32,
	user_id: String,
	parent_id: Option<String>,
}

async fn get_user_sets(
	_: State<AppState>,
	Conn(conn): Conn,
	Path(id): Path<String>,
) -> Result<Json<Vec<UserEmoteSet>>> {
	if !user_exists(&conn, &id).await {
		return Err(JsonError::UnknownUser.into());
	}

	let sets = conn
		.query(
			"
			SELECT sets.*
			FROM
				users
				JOIN sets ON users.id = sets.user_id
			WHERE user_id = $1
			ORDER BY id
			",
			&[&id],
		)
		.await?
		.into_iter()
		.map(|row| row.into())
		.collect();

	Ok(Json(sets))
}

async fn get_user_channel_set(
	_: State<AppState>,
	Conn(conn): Conn,
	Path(id): Path<String>,
) -> Result<Json<UserEmoteSet>> {
	let set = conn
		.query_one(
			"
			SELECT sets.*
			FROM
				users
				JOIN sets ON users.channel_set_id = sets.id
			WHERE users.id = $1
			",
			&[&id],
		)
		.await?
		.into();

	Ok(Json(set))
}

async fn user_exists(conn: &Connection, id: &String) -> bool {
	conn.query_one("SELECT EXISTS (SELECT id FROM users WHERE id = $1)", &[&id])
		.await
		.unwrap()
		.get(0)
}