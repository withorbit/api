use axum::Router;

use crate::AppState;

pub mod admin;
pub mod colors;
pub mod emotes;
pub mod sets;
pub mod users;

pub fn router(state: &AppState) -> Router<AppState> {
	Router::new()
		.merge(self::admin::router())
		.merge(self::colors::router())
		.merge(self::emotes::router(state))
		.merge(self::sets::router(state))
		.merge(self::users::router(state))
}
