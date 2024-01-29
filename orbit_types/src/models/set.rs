use orbit_macros::{FromJsonb, FromRow};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use super::emote::Emote;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct EmoteSet {
	#[serde_as(serialize_as = "DisplayFromStr")]
	id: i64,
	name: String,
	capacity: i32,

	#[serde_as(serialize_as = "DisplayFromStr")]
	user_id: i64,

	#[serde_as(serialize_as = "Option<DisplayFromStr>")]
	parent_id: Option<i64>,
}

#[serde_as]
#[derive(Debug, Serialize, FromRow)]
pub struct EmoteSetWithEmotes {
	#[serde_as(serialize_as = "DisplayFromStr")]
	id: i64,
	name: String,
	capacity: i32,

	#[serde_as(serialize_as = "DisplayFromStr")]
	user_id: i64,

	#[serde_as(serialize_as = "Option<DisplayFromStr>")]
	parent_id: Option<i64>,
	emotes: EmoteVec,
}

#[derive(Debug, Deserialize, Serialize, FromJsonb)]
struct EmoteVec(Vec<Emote>);

#[derive(Debug, Deserialize)]
pub struct CreateEmoteSet {
	pub name: String,
	pub capacity: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmoteSet {
	pub name: Option<String>,
	pub capacity: Option<i32>,
}
