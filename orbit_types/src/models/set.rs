use orbit_macros::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct EmoteSet {
	pub id: String,
	pub name: String,
	pub capacity: i32,
	pub user_id: String,
	pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EmoteSetWithEmotes {
	pub id: String,
	pub name: String,
	pub capacity: i32,
	pub user_id: String,
	pub parent_id: Option<String>,
	pub emotes: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmoteSet {
	pub name: String,
	pub capacity: i32,
}
