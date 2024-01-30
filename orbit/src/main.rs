mod auth;
mod db;
mod error;
mod routes;

use std::time::Duration;

use axum::http::Request;
use axum::Router;
use shuttle_secrets::SecretStore;
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

use crate::error::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone)]
struct AppState {
	s3: aws_sdk_s3::Client,
	pool: db::Pool,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
	dotenvy::dotenv().expect("Loading `.env` failed");

	tracing_subscriber::registry()
		.with(EnvFilter::new("orbit=debug,tower_http=debug"))
		.with(fmt::layer())
		.init();

	let database_url = get_secret(&secrets, "DATABASE_URL");
	let s3_config = aws_config::load_from_env().await;

	let pool = db::init_db(database_url).await;

	let app_state = AppState {
		s3: aws_sdk_s3::Client::new(&s3_config),
		pool,
	};

	let router = Router::new()
		.nest("/api", routes::router(&app_state))
		.layer(
			ServiceBuilder::new()
				.layer(
					TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
						tracing::info_span!(
							"http_request",
							"{} {}",
							request.method(),
							request.uri()
						)
					}),
				)
				.layer(TimeoutLayer::new(Duration::from_secs(120))),
		)
		.with_state(app_state);

	Ok(router.into())
}

fn get_secret(secrets: &SecretStore, key: &str) -> String {
	secrets.get(key).expect(&format!("`{key}` not set."))
}
