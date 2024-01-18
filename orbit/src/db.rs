use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

use crate::error::Error;
use crate::AppState;

pub type Pool = bb8::Pool<PostgresConnectionManager<NoTls>>;

pub async fn init_db(url: String) -> Pool {
	let manager = PostgresConnectionManager::new_from_stringlike(url, NoTls)
		.expect("Invalid connection string");
	let pool = bb8::Pool::builder().build(manager).await.unwrap();

	pool
}

pub type Connection = bb8::PooledConnection<'static, PostgresConnectionManager<NoTls>>;

pub struct Conn(pub Connection);

#[axum::async_trait]
impl<S> FromRequestParts<S> for Conn
where
	Pool: FromRef<S>,
	S: Send + Sync,
{
	type Rejection = Error;

	async fn from_request_parts(_: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let pool = Pool::from_ref(state);
		let conn = pool
			.get_owned()
			.await
			.expect("Failed to retrieve database connection");

		Ok(Self(conn))
	}
}

#[axum::async_trait]
impl FromRequestParts<AppState> for Conn {
	type Rejection = Error;

	async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
		let conn = state
			.pool
			.get_owned()
			.await
			.expect("Failed to retrieve database connection");

		Ok(Self(conn))
	}
}
