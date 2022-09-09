#![allow(dead_code)]

#[path = "serde_bytes_array.rs"]
mod serde_bytes_array;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PingQuery {
	#[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
}

impl PingQuery {
	pub fn into_bytes(self) -> Vec<u8> {
		Query { t: "0".into(), v: "XX01", y: "q", q: "ping", a: self }.to_bytes()
	}
}

#[derive(Debug, Serialize)]
pub struct GetPeersQuery {
	#[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
	#[serde(with = "serde_bytes_array")]
	pub info_hash: [u8; 20],
}

impl GetPeersQuery {
	pub fn into_bytes(self) -> Vec<u8> {
		Query { t: self.info_hash.into(), v: "XX01", y: "q", q: "get_peers", a: self }.to_bytes()
	}
}

#[derive(Debug, Serialize)]
pub struct FindNodeQuery {
	#[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
	#[serde(with = "serde_bytes_array")]
	pub target: [u8; 20],
}

impl FindNodeQuery {
	pub fn into_bytes(self) -> Vec<u8> {
		Query { t: "0".into(), v: "XX01", y: "q", q: "find_node", a: self }.to_bytes()
	}
}

#[derive(Debug, Serialize)]
pub struct SampleInfohashesQuery {
	#[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
	#[serde(with = "serde_bytes_array")]
	pub target: [u8; 20],
}

impl SampleInfohashesQuery {
	pub fn into_bytes(self) -> Vec<u8> {
		Query { t: "0".into(), v: "XX01", y: "q", q: "sample_infohashes", a: self }.to_bytes()
	}
}

#[derive(Debug, Deserialize)]
pub struct CompactInfo {
	// #[serde(with = "serde_bytes_array")]
	pub id: [u8; 20],
	// #[serde(with = "serde_bytes_array")]
	pub ip: [u8; 4],
	// #[serde(with = "serde_bytes_array")]
	port: [u8; 2],
}

impl CompactInfo {
	pub fn ip_string(&self) -> String {
		format!("{}.{}.{}.{}", self.ip[0], self.ip[1], self.ip[2], self.ip[3])
	}
	pub fn port(&self) -> u16 {
		u16::from_be_bytes(self.port)
	}
}

#[derive(Debug, Serialize)]
struct Query<T> {
	#[serde(with = "serde_bytes")]
	t: Vec<u8>,
	v: &'static str,
	y: &'static str,
	q: &'static str,
	a: T,
}

impl<T: Serialize> Query<T> {
	fn to_bytes(&self) -> Vec<u8> {
		serde_bencode::to_bytes(&self).unwrap()
	}
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Bytes {
	#[serde(with = "serde_bytes")]
	Bytes(Vec<u8>),
}

#[derive(Debug, Deserialize)]
pub struct ResponseArgs {
	#[serde(with = "serde_bytes")]
	pub id: Vec<u8>,
	pub token: Option<Bytes>,
	pub nodes: Option<Bytes>,
	pub samples: Option<Bytes>,
	pub interval: Option<i64>,
	pub num: Option<i64>,
}

pub struct ParsedResponseArgs {
	pub id: [u8; 20],
	pub token: Option<Vec<u8>>,
	pub nodes: Option<Vec<CompactInfo>>,
	pub samples: Option<Vec<[u8; 20]>>,
	pub interval: Option<i64>,
	pub num: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Response {
	#[serde(with = "serde_bytes")]
	pub t: Vec<u8>,
	// y: &'static str,
	// q: &'static str,
	r: ResponseArgs,
}

impl<'a> Response {
	pub fn from_bytes(buf: &'a [u8]) -> Result<ResponseArgs, serde_bencode::Error> {
		let x = serde_bencode::de::from_bytes::<Response>(buf);
		let x = x?;
		Ok(x.r)
	}
}
