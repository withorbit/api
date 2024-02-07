use axum::extract::Json;
use axum::routing::post;
use axum::Router;
use orbit_types::models::user::{Color, CreateColor, Role};
use orbit_types::Snowflake;

use crate::auth::AuthUser;
use crate::db::Conn;
use crate::error::{JsonError, ResultExt};
use crate::{AppState, Result};

pub fn router() -> Router<AppState> {
	Router::new()
}
