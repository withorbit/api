use axum::Router;

use crate::AppState;

pub mod channels;
pub mod emotes;
pub mod sets;
pub mod users;

pub fn router() -> Router<AppState> {
	Router::new()
		.merge(self::channels::router())
		.merge(self::emotes::router())
		.merge(self::sets::router())
		.merge(self::users::router())
}
