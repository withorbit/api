use axum::extract::{Json, Path, State};
use axum::routing::{delete, get, patch, post};
use axum::Router;
use serde::{Deserialize, Serialize};

use crate::{AppState, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct EmoteSet {}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/sets", post(create_set))
		.route("/sets/:id", get(get_set))
		.route("/sets/:id", patch(update_set))
		.route("/sets/:id", delete(delete_set))
}

async fn create_set(State(state): State<AppState>) -> Result<Json<EmoteSet>> {
	todo!()
}

async fn get_set(Path(id): Path<String>) {
	todo!()
}

async fn update_set(Path(id): Path<String>) {
	todo!()
}

async fn delete_set(Path(id): Path<String>) {
	todo!()
}
