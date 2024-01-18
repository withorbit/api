use std::sync::atomic::{AtomicI64, Ordering};
use std::time::SystemTime;

static INCREMENT: AtomicI64 = AtomicI64::new(0);

// increment = (increment + 1) & 4095
fn next_increment() -> i64 {
	INCREMENT.fetch_add(1, Ordering::Relaxed);
	INCREMENT.fetch_and(4095i64, Ordering::Relaxed);

	INCREMENT.load(Ordering::Relaxed)
}

#[derive(Debug, Clone)]
pub struct Snowflake(pub String);

impl Snowflake {
	pub const EPOCH: i64 = 1_696_118_400_000;

	pub fn new() -> Self {
		let increment = next_increment();

		let now = SystemTime::now();
		let now_ms = now
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("Time went backwards")
			.as_millis() as i64;

		let id = ((now_ms - Self::EPOCH) << 22) | (increment & 4095i64);
		Self(id.to_string())
	}
}
