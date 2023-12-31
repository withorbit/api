use axum::extract::Path;
use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
	Router::new().route("/channels/:id", get(get_channel))
}

async fn get_channel(Path(id): Path<String>) {
	todo!()
}
