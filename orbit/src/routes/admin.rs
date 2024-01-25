use axum::extract::Json;
use axum::routing::post;
use axum::Router;
use orbit_types::models::user::{Color, CreateColor, Role};
use orbit_types::snowflake::Snowflake;

use crate::auth::AuthUser;
use crate::db::Conn;
use crate::error::{JsonError, ResultExt};
use crate::{AppState, Result};

pub fn router() -> Router<AppState> {
	Router::new().route("/admin/colors", post(create_color))
}

async fn create_color(
	Conn(conn): Conn,
	user: AuthUser,
	Json(body): Json<CreateColor>,
) -> Result<Json<Color>> {
	if !user.roles.contains(&Role::Admin) {
		return Err(JsonError::Forbidden.into());
	}

	let color = conn
		.query_one(
			"
			INSERT INTO colors (id, name, gradient, shadow)
			VALUES ($1, $2, $3, $4)
			",
			&[
				&Snowflake::new().0,
				&body.name,
				&body.gradient,
				&body.shadow,
			],
		)
		.await
		.on_constraint("colors_name_key", JsonError::ColorExists.into())?
		.into();

	Ok(Json(color))
}
