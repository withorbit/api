use orbit_macros::FromRow;
use serde::{Deserialize, Serialize};

use super::user::User;

#[derive(Debug, Deserialize, Serialize, FromRow)]
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
	pub user_id: String,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct EmoteWithUser {
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
	pub user: User,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEmote {
	pub name: String,
	pub tags: Vec<String>,
	pub width: i32,
	pub height: i32,
	pub public: bool,
	pub animated: bool,
	pub modifier: bool,
	pub nsfw: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmote {
	pub approved: Option<bool>,
	pub nsfw: Option<bool>,
}
