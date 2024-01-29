use orbit_macros::{FromJsonb, FromRow};
use serde::{Deserialize, Serialize};
use tokio_postgres::types::{FromSql, ToSql};

#[derive(Debug, Deserialize, Serialize, PartialEq, ToSql, FromSql)]
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

#[derive(Debug, Deserialize, Serialize, FromJsonb, FromRow)]
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

#[derive(Deserialize, Serialize, FromRow)]
pub struct UserEmote {
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

#[derive(Deserialize, Serialize, FromRow)]
pub struct UserEmoteSet {
	id: String,
	name: String,
	capacity: i32,
	user_id: String,
	parent_id: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct Color {
	id: String,
	name: String,
	gradient: String,
	shadow: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateColor {
	pub name: String,
	pub gradient: String,
	pub shadow: String,
}
