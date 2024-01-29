use orbit_macros::{FromJsonb, FromRow};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
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

#[serde_as]
#[derive(Debug, Deserialize, Serialize, FromJsonb, FromRow)]
pub struct User {
	#[serde_as(serialize_as = "DisplayFromStr")]
	pub id: i64,
	pub twitch_id: i32,
	pub username: String,
	pub avatar_url: String,
	pub roles: Vec<Role>,
	pub badge_url: Option<String>,

	#[serde_as(serialize_as = "Option<DisplayFromStr>")]
	pub color_id: Option<i64>,

	#[serde_as(serialize_as = "DisplayFromStr")]
	pub channel_set_id: i64,
}

#[serde_as]
#[derive(Deserialize, Serialize, FromRow)]
pub struct UserEmoteSet {
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
#[derive(Serialize, FromRow)]
pub struct Color {
	#[serde_as(serialize_as = "DisplayFromStr")]
	id: i64,
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
