use orbit_macros::{FromJsonb, FromRow};
use serde::{Deserialize, Serialize};

use super::emote::Emote;

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct EmoteSet {
	id: String,
	name: String,
	capacity: i32,
	user_id: String,
	parent_id: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EmoteSetWithEmotes {
	id: String,
	name: String,
	capacity: i32,
	user_id: String,
	parent_id: Option<String>,
	emotes: EmoteVec,
}

#[derive(Debug, Deserialize, Serialize, FromJsonb)]
struct EmoteVec(Vec<Emote>);

#[derive(Debug, Deserialize)]
pub struct CreateEmoteSet {
	pub name: String,
	pub capacity: i32,
}
