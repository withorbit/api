use rand::distributions::Alphanumeric;
use rand::Rng;
use tokio_postgres::{Error, NoTls};

#[tokio::main]
async fn main() -> Result<(), Error> {
	let url = "postgresql://postgres@localhost:5432/postgres";
	let (mut client, connection) = tokio_postgres::connect(url, NoTls).await?;

	tokio::spawn(async move {
		if let Err(err) = connection.await {
			eprintln!("Failed to connect: {err}");
		}
	});

	// using rand instead of snowflake to simulate gaps of time between creation
	let (
		user_id_1,
		user_id_2,
		channel_set_id_1,
		channel_set_id_2,
		emote_id_v1,
		emote_id_v2,
		emote_id_other,
	) = (
		gen_id(),
		gen_id(),
		gen_id(),
		gen_id(),
		gen_id(),
		gen_id(),
		gen_id(),
	);

	let transaction = client.transaction().await?;

	transaction
		.execute(
			&format!(
				"
				INSERT INTO
					users (
						id, twitch_id, username, avatar_url,
						roles, badge_url, color_id, channel_set_id
					)
				VALUES
					(
						{user_id_1}, 123456789, 'xiBread_', 'https://placehold.co/300,
						'{{admin}}'::role[], null, null, {channel_set_id_1}
					),
					(
						{user_id_2}, 987654321, 'john_doe', 'https://placehold.co/300,
						'{{}}', null, null, {channel_set_id_2}
					)
				"
			),
			&[],
		)
		.await?;

	transaction
		.execute(
			&format!(
				"
				INSERT INTO
					sets (id, name, capacity, user_id, parent_id)
				VALUES
					({user_id_1}, 'Personal', 15, {user_id_1}, null),
					({channel_set_id_1}, 'Channel', 500, {user_id_1}, null),
					($1, 'Random', 1000, {user_id_1}, null),
					({user_id_2}, 'Personal', 15, {user_id_2}, null),
					({channel_set_id_2}, 'Channel', 500, {user_id_2}, null),
					($2, 'Cats', 1000, {user_id_2}, null)
				"
			),
			&[&gen_id(), &gen_id()],
		)
		.await?;

	// second emote looks like a duplicate without the url but we're mimicking
	// a different image being uploaded, hence the different id
	transaction
		.execute(
			&format!(
				r#"
				INSERT INTO
					emotes (
						id, name, tags, width, height, approved,
						public, animated, modifier, nsfw, user_id
					)
				VALUES
					(
						{emote_id_v1}, 'KEKW', '{{"funny", "laughing"}}', 28, 28,
						false, true, true, false, false, {user_id_1}
					),
					(
						{emote_id_v2}, 'KEKW', '{{"funny", "laughing"}}', 28, 28,
						false, true, true, false, false, {user_id_1}
					),
					(
						{emote_id_other}, 'Pepega', '{{"pepe", "stupid"}}', 28, 28,
						false, true, false, false, false, {user_id_1}
					)
				"#
			),
			&[],
		)
		.await?;

	transaction
		.execute(
			&format!(
				"
				INSERT INTO
					versions (id, name, description, emote_id)
				VALUES
					($1, 'Initial upload', 'john_doe uploaded KEKW', {emote_id_v1}),
					($2, 'Higher quality', 'Fixed pixelation', {emote_id_v2}),
				"
			),
			&[&gen_id(), &gen_id()],
		)
		.await?;

	transaction
		.execute(
			&format!(
				"
				INSERT INTO
					emotes_to_sets (emote_id, set_id),
				VALUES
					({emote_id_v1}, {channel_set_id_1}),
					({emote_id_other}, {channel_set_id_1})
				"
			),
			&[],
		)
		.await?;

	let session_id = rand::thread_rng()
		.sample_iter(&Alphanumeric)
		.take(40)
		.map(char::from)
		.collect::<String>();

	transaction
		.execute(
			"
			INSERT INTO
				sessions (id, expires_at, user_id)
			VALUES
				($1, '2024-01-01 12:59:30.111', $2)
			",
			&[&session_id, &user_id_1],
		)
		.await?;

	transaction.commit().await?;

	Ok(())
}

fn gen_id() -> i64 {
	rand::thread_rng().gen_range(4e17f64..=5e17f64).round() as i64
}
