use orbit_macros::{FromJsonb, FromRow};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use super::user::User;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, FromJsonb, FromRow)]
pub struct Emote {
	#[serde_as(serialize_as = "DisplayFromStr")]
	id: i64,
	name: String,
	tags: Vec<String>,
	width: i32,
	height: i32,
	approved: bool,
	public: bool,
	animated: bool,
	modifier: bool,
	nsfw: bool,

	#[serde_as(serialize_as = "DisplayFromStr")]
	user_id: i64,

	#[serde_as(serialize_as = "Vec<DisplayFromStr>")]
	versions: Vec<i64>,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct EmoteWithUser {
	#[serde_as(serialize_as = "DisplayFromStr")]
	id: i64,
	name: String,
	tags: Vec<String>,
	width: i32,
	height: i32,
	approved: bool,
	public: bool,
	animated: bool,
	modifier: bool,
	nsfw: bool,

	#[serde_as(serialize_as = "Vec<DisplayFromStr>")]
	versions: Vec<i64>,
	user: User,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct EmoteVersion {
	#[serde_as(serialize_as = "DisplayFromStr")]
	id: i64,
	name: String,
	description: String,

	#[serde_as(serialize_as = "DisplayFromStr")]
	emote_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct GetEmoteQuery {
	#[serde(rename(deserialize = "v"))]
	pub version: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchEmotesQuery {
	#[serde(rename(deserialize = "q"))]
	pub query: Option<String>,
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
