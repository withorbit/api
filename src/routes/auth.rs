use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use rand::distributions::{Alphanumeric, DistString};

#[derive(Clone)]
pub struct AuthState {
	pub twitch_client_id: String,
	pub twitch_client_secret: String,
}

pub async fn login(State(state): State<AuthState>) -> impl IntoResponse {
	let state_param = Alphanumeric.sample_string(&mut rand::thread_rng(), 43);
	let state_cookie = format!(
		"twitch_oauth_state={}; Path=/; Max-Age=3600; HttpOnly; Secure;",
		state_param
	);

	let auth_url = format!(
		"https://id.twitch.tv/oauth2/authorize \
		?response_type=code \
		&client_id={} \
		&state={} \
		&redirect_url=http://localhost:5173/login/twitch/callback",
		state.twitch_client_id, state_param
	);

	(
		StatusCode::FOUND,
		[
			(header::SET_COOKIE, state_cookie),
			(header::LOCATION, auth_url.to_string()),
		],
	)
}

// pub async fn callback()
