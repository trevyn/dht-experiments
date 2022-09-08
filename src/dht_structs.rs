#[path = "serde_bytes_array.rs"]
mod serde_bytes_array;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PingQuery {
	#[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
}

impl PingQuery {
	pub fn into_bytes(self) -> Vec<u8> {
		Query { y: "q", q: "ping", a: self }.to_bytes()
	}
}

#[derive(Debug, Serialize)]
struct Query<T> {
	y: &'static str,
	q: &'static str,
	a: T,
}

impl<T: Serialize> Query<T> {
	fn to_bytes(&self) -> Vec<u8> {
		serde_bencode::to_bytes(&self).unwrap()
	}
}
