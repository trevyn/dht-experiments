#![allow(dead_code)]

#[path = "serde_bytes_array.rs"]
mod serde_bytes_array;

use log::*;
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

#[derive(Debug, bincode::Decode)]
pub struct CompactInfo {
	pub id: [u8; 20],
	pub ip: [u8; 4],
	port: [u8; 2],
}

impl CompactInfo {
	pub fn host(&self) -> String {
		format!("{}:{}", self.ip_string(), self.port())
	}
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

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Bytes {
	#[serde(with = "serde_bytes")]
	Bytes(Vec<u8>),
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResponseArgs {
	#[serde(with = "serde_bytes")]
	pub id: Vec<u8>,
	pub token: Option<Bytes>,
	pub nodes: Option<Bytes>,
	pub values: Option<Vec<Bytes>>,
	pub samples: Option<Bytes>,
	pub interval: Option<i64>,
	pub num: Option<i64>,
}

impl ResponseArgs {
	pub fn id(&self) -> Result<[u8; 20], Vec<u8>> {
		self.id.clone().try_into()
	}
	pub fn nodes(&self) -> Vec<CompactInfo> {
		match self.nodes {
			None => Vec::new(),
			Some(Bytes::Bytes(ref bytes)) => bytes
				.chunks_exact(26)
				.filter_map(|chunk| {
					bincode::decode_from_slice(
						chunk,
						bincode::config::standard().skip_fixed_array_length().with_limit::<26>(),
					)
					.map_err(|e| {
						warn!("deserialize nodes error: {e:?}");
						e
					})
					.map(|r| r.0)
					.ok()
				})
				.collect(),
		}
	}
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
