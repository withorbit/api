mod error;
mod routes;
mod snowflake;

use std::time::Duration;

use axum::routing::get;
use axum::Router;
use routes::auth::AuthState;
use shuttle_secrets::SecretStore;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone)]
struct AppState {
	s3: aws_sdk_s3::Client,
	pool: sqlx::PgPool,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
	dotenvy::dotenv().expect("Loading .env failed");

	tracing_subscriber::registry()
		.with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "api=debug".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	let database_url = get_secret(&secrets, "DATABASE_URL");
	let config = aws_config::load_from_env().await;

	let pool = PgPoolOptions::new()
		.max_connections(50)
		.connect(&database_url)
		.await
		.expect("Failed to connect to database");

	let app_state = AppState {
		s3: aws_sdk_s3::Client::new(&config),
		pool,
	};

	let auth_state = AuthState {
		twitch_client_id: get_secret(&secrets, "TWITCH_CLIENT_ID"),
		twitch_client_secret: get_secret(&secrets, "TWITCH_CLIENT_SECRET"),
	};

	let router = Router::new()
		.route(
			"/login/twitch",
			get(routes::auth::login).with_state(auth_state),
		)
		.nest("/api", routes::router())
		.layer(
			ServiceBuilder::new()
				.layer(TraceLayer::new_for_http())
				.layer(TimeoutLayer::new(Duration::from_secs(120))),
		)
		.with_state(app_state);

	Ok(router.into())
}

fn get_secret(secrets: &SecretStore, key: &str) -> String {
	secrets.get(key).expect(&format!("`{key}` not set."))
}
