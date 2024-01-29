use orbit_macros::{FromJsonb, FromRow};
use serde::{Deserialize, Serialize};

use super::user::User;

#[derive(Debug, Deserialize, Serialize, FromJsonb, FromRow)]
pub struct Emote {
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

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct EmoteWithUser {
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
	user: User,
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
